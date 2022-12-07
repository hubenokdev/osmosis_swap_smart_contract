#[cfg(not(feature = "library"))]
use cosmwasm_std::{ DepsMut, MessageInfo, Response, BankMsg,  Coin, SubMsg, Addr, Env
    , SubMsgResult, Reply, Uint128, CosmosMsg, WasmMsg, to_binary, QuerierWrapper};
use cw_osmo_proto::osmosis::gamm::v1beta1::{ MsgSwapExactAmountIn, MsgSwapExactAmountInResponse, SwapAmountInRoute as Osmo_SwapAmountInRoute };
use cw_osmo_proto::cosmos::base::v1beta1::{ Coin as Osmo_Coin };
use cw_osmo_proto::proto_ext::{MessageExt};
use cw20::{Denom, Cw20ExecuteMsg};

use crate::error::ContractError;
use crate::state::{SWAP_TO_REPLY_STATES, SwapMsgReplyState, State, config, config_read, BOT_ROLES};
use crate::helpers;
const ATOM_DENOM: &str = "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"; //ibc atom token on osmo

pub fn execute_transfer(deps: DepsMut, _info: MessageInfo, addr: String) -> Result<Response, ContractError> {
    let to_addr = match deps.api.addr_validate(addr.clone().as_str()).ok() {
        Some(x) => x,
        None => return Err(ContractError::Unauthorized {}),
    };

    //let to_addr = addr;
    let mut sent_funds: Vec<Coin> = Vec::new();
    sent_funds.push(Coin::new(140000, "uosmo"));// info.funds.clone();
    let msg = BankMsg::Send {
        to_address: to_addr.into(),
        amount: sent_funds,
    };

    Ok(Response::new()
        .add_attribute("method", "execute_transfer")
        .add_message(msg)
    )
}

pub fn buy_token(
    deps: DepsMut, 
    state: &mut State,
    info: MessageInfo, 
    env: Env,
    osmo_amount: Uint128,
    pool_id: u64, 
    denom_out_token: String, 
    token_amount_per_native: Uint128,
    slippage_bips: Uint128,
    platform_fee_bips: Uint128,
    gas_estimate: Uint128,
    deadline: u64,
    recipient: Addr, 
    reply_id: u64
) -> Result<Response, ContractError> {
    if !BOT_ROLES.has(deps.storage, info.sender.clone()) {
        return Err(ContractError::Unauthorized {});    
    }
    let enabled = BOT_ROLES.load(deps.storage, info.sender)?;
    if !enabled {
        return Err(ContractError::UnauthorizedRole {});    
    }

    if env.block.time.seconds() > deadline {
        return Err(ContractError::Expired { });
    }

    if slippage_bips > Uint128::from(10000u128) {
        return Err(ContractError::BuyingUtilityOverSlippages { });
    }

    if gas_estimate > osmo_amount {
        return Err(ContractError::InsufficientToken{});
    }

    let swap_atom_msgs = get_message_swap_atom_to_osmo(deps.querier, env.contract.address.clone())?;

    let mut _osmo_amount = osmo_amount - gas_estimate;
    let platform_fee = platform_fee_bips * osmo_amount / Uint128::from(10000u128);
    //state.pending_platform_fee += platform_fee;

    let amount_out_min = _osmo_amount * token_amount_per_native * (Uint128::from(10000u128) - slippage_bips) / Uint128::from(10000000000u128);
    _osmo_amount -= platform_fee;

    //let funds = info.funds.clone().pop().unwrap();
    let in_coin = Osmo_Coin {
        denom: String::from("uosmo"),
        amount: osmo_amount.to_string()
    };

    let mut osmo_routes: Vec<Osmo_SwapAmountInRoute> = Vec::new();
    osmo_routes.push(Osmo_SwapAmountInRoute {
        pool_id,
        token_out_denom: denom_out_token.clone(),
    });

    let token_out_min_amount: String = amount_out_min.to_string(); ////

    let msg_buytoken = MsgSwapExactAmountIn {
        sender: env.contract.address.into_string(),
        routes: osmo_routes,
        token_in: Option::from(in_coin),
        token_out_min_amount,
    };

    let data = SwapMsgReplyState{denom_out: denom_out_token
                                            , to: recipient
                                            , platformfee: state.pending_platform_fee + platform_fee};

    SWAP_TO_REPLY_STATES.save(
        deps.storage,
        reply_id,
        &data
    )?;

    let msg_buytoken = msg_buytoken.to_msg()?;

    Ok(Response::new().add_messages(swap_atom_msgs)
        .add_submessage(SubMsg::reply_on_success(msg_buytoken, reply_id))
        .add_attribute("method", "execute_swap")
    )
}

pub fn try_set_admin(
    deps: DepsMut,
    state: &mut State,
    info: MessageInfo,
    new_admin: Addr
) -> Result<Response, ContractError> {
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized { });
    }

    state.owner = new_admin.clone();
    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("new_admin", new_admin)
    )
}

pub fn try_set_bot_role(
    deps: DepsMut,
    state: State,
    info: MessageInfo,
    new_bot: Addr,
    role: bool
) -> Result<Response, ContractError> {
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized { });
    }

    BOT_ROLES.save(deps.storage, new_bot.clone(), &role)?;
    
    Ok(Response::new()
        .add_attribute("bot added", "Yes")
    )
}

pub fn try_withdraw_fee(
    deps: DepsMut,
    state: &mut State,
    info: MessageInfo,
    to: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized { });
    }

    state.pending_platform_fee -= amount;

    config(deps.storage).save(&state)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    msgs.push(transfer_token_message(Denom::Native(String::from("uosmo")), amount, to)?);

    Ok(Response::new()
        .add_messages(msgs)
    )
}

pub fn transfer_token_message(
    denom: Denom,
    amount: Uint128,
    receiver: Addr
) -> Result<CosmosMsg, ContractError> {

    match denom.clone() {
        Denom::Native(native_str) => {
            return Ok(BankMsg::Send {
                to_address: receiver.clone().into(),
                amount: vec![Coin{
                    denom: native_str,
                    amount
                }]
            }.into());
        },
        Denom::Cw20(cw20_address) => {
            return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_address.clone().into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: receiver.clone().into(),
                    amount
                })?,
            }));
        }
    }
}

pub fn handle_swap_reply(
    deps: DepsMut,
    msg: Reply,
    send_to: SwapMsgReplyState,
) -> Result<Response, ContractError> {

    match msg.result {
        SubMsgResult::Ok(tx) => {
            let gamm_res = helpers::parse_gamm_result::<MsgSwapExactAmountInResponse>(tx);
                
            match gamm_res {
                Ok(amount_out) => {
                    let mut new_coin = Vec::new();
                    new_coin.push(Coin::new(amount_out, send_to.denom_out));

                    let bank_msg = BankMsg::Send {
                        to_address: send_to.to.into_string(),
                        amount: new_coin,
                    };

                    let mut state = config_read(deps.storage).load()?;

                    state.pending_platform_fee += send_to.platformfee;

                    config(deps.storage).save(&state)?;                    
                        
                    return Ok(Response::new()
                        .add_message(bank_msg)
                        .add_attribute("token_out_amount", Uint128::from(amount_out)));
                }
                Err(err) => {
                    Err(err)
                }
            }
        }
        SubMsgResult::Err(err) => {
            Err(ContractError::FailedSwap {
                reason: err,
            })
        }
    }    
}

fn get_message_swap_atom_to_osmo(    
    querier: QuerierWrapper,
    sender: Addr,
)-> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];    
    //let msg_buytoken: MsgSwapExactAmountIn ;

    let token_balance = helpers::get_token_amount(querier, Denom::Native(String::from(ATOM_DENOM))
        , sender.clone())?;

    if token_balance == Uint128::zero() {
        return Ok(messages);
    }

    let in_coin = Osmo_Coin {
        denom: String::from(ATOM_DENOM),
        amount: token_balance.to_string()
    };

    let amount_out_min = 1000;
    let pool_id: u64 = 1;

    let mut osmo_routes: Vec<Osmo_SwapAmountInRoute> = Vec::new();
    osmo_routes.push(Osmo_SwapAmountInRoute {
        pool_id,
        token_out_denom: String::from("uosmo"),
    });

    let token_out_min_amount: String = amount_out_min.to_string(); ////

    let msg_buytoken = MsgSwapExactAmountIn {
        sender: sender.into_string(),
        routes: osmo_routes,
        token_in: Option::from(in_coin),
        token_out_min_amount,
    };  

    messages.push(msg_buytoken.to_msg()?);
    Ok(messages)
}