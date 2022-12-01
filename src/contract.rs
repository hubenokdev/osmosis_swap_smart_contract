#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, Reply, Uint128, StdResult, Binary, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, AllInfos};
use crate::state::{SWAP_TO_REPLY_STATES, config_read, config, State};
use crate::execute::{execute_transfer, buy_token, handle_swap_reply, try_set_admin, try_set_bot_role, try_withdraw_fee};
use crate::helpers;
use cw20::Denom;

// version info for migration info
const CONTRACT_NAME: &str = "osmo.trade.bot";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// packets live one hour
pub const PACKET_LIFETIME: u64 = 60 * 60;
pub const SWAP_REPLY_ID: u64 = 1u64;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        owner: _info.sender.clone(),
        pending_platform_fee: Uint128::zero(),
    };

    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut state = config_read(deps.storage).load()?;
    match msg {
        ExecuteMsg::SetAdmin { new_admin } => try_set_admin(deps, &mut state, info, new_admin),
        ExecuteMsg::SetBotRole { new_bot, enabled } => try_set_bot_role(deps, state, info, new_bot, enabled),
        ExecuteMsg::BuyToken {osmo_amount, pool_id, denom_token, token_amount_per_native, slippage_bips
            , recipient, platform_fee_bips, gas_estimate, deadline} => 
                buy_token(deps
                    , &mut state
                    , info
                    , env
                    , osmo_amount
                    , pool_id
                    , denom_token
                    , token_amount_per_native
                    , slippage_bips
                    , platform_fee_bips
                    , gas_estimate
                    , deadline.u64()
                    , recipient
                    , SWAP_REPLY_ID),      

        ExecuteMsg::WithdrawFee { to, amount } => try_withdraw_fee(deps, &mut state, info, to, amount),

        ExecuteMsg::Transfer { address } => execute_transfer(deps, info, address),
        //ExecuteMsg::Swap { pool_id, token_out_denom, token_out_min_amount, to } => execute_swap(deps, env.contract.address.into(), info, pool_id, token_out_denom, token_out_min_amount, to),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id == SWAP_REPLY_ID {
        // get intermediate swap reply state. Error if not found.
        let swap_infos = SWAP_TO_REPLY_STATES.load(deps.storage, msg.id)?;

        // prune intermedate state
        SWAP_TO_REPLY_STATES.remove(deps.storage, msg.id);

        // call reply function to handle the swap return
        handle_swap_reply(deps, msg, swap_infos)
    } else {
        Ok(Response::new()
            .add_attribute("Default", "reply"))
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInfos {token} => to_binary(&query_infos(deps, env, token)?),
    }
}

fn query_infos(deps: Deps, env: Env, token: String) -> StdResult<AllInfos> {
    let state = config_read(deps.storage).load()?;
    let admin = state.owner;
    let pending_platform_fee = state.pending_platform_fee;
    let blocktime = env.block.time.seconds();
    let contract_address = env.contract.address.clone();
    let token_balance = helpers::get_token_amount(deps.querier, Denom::Native(token), env.contract.address.clone())?;
    let token_balances = helpers::get_tokens_amounts(deps.querier, env.contract.address)?;
    
    Ok(AllInfos { admin, pending_platform_fee, blocktime, token_balance, token_balances, contract_address })
}
