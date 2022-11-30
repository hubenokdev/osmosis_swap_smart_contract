#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point,  DepsMut, Env, MessageInfo, Response, BankMsg,  Coin, SubMsg, Addr
    , SubMsgResult, SubMsgResponse, Reply, Uint128, };
use cw2::set_contract_version;
use cw_osmo_proto::osmosis::gamm::v1beta1::{ MsgSwapExactAmountIn, MsgSwapExactAmountInResponse, SwapAmountInRoute as Osmo_SwapAmountInRoute };
use cw_osmo_proto::cosmos::base::v1beta1::{ Coin as Osmo_Coin };
use cw_osmo_proto::proto_ext::{MessageExt};
use cw_osmo_proto::proto_ext::proto_decode;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{SWAP_TO_REPLY_STATES, SwapMsgReplyState};

// version info for migration info
const CONTRACT_NAME: &str = "osmo.trade.bot";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// packets live one hour
pub const PACKET_LIFETIME: u64 = 60 * 60;
pub const SWAP_REPLY_ID: u64 = 1u64;

pub trait GammResult {
    fn amount(&self) -> &String;
}

impl GammResult for MsgSwapExactAmountInResponse {
    fn amount(&self) -> &String {
        &self.token_out_amount
    }
}

pub fn parse_coin(value: &str) -> Result<Coin, ContractError> {
    let mut num_str = vec![];
    for c in value.chars() {
        if !c.is_numeric() {
            break;
        }

        num_str.push(c)
    }

    let amount_str: String = num_str.into_iter().collect();
    let amount = amount_str
        .parse::<u128>()
        .map_err(|_| ContractError::InvalidAmountValue {})?;
    let denom = value.replace(amount_str.as_str(), "");

    Ok(Coin {
        amount: amount.into(),
        denom,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { address } => execute_transfer(deps, info, address),
        ExecuteMsg::Swap { pool_id, token_out_denom, token_out_min_amount, to } => execute_swap(deps, _env.contract.address.into(), info, pool_id, token_out_denom, token_out_min_amount, to),
    }
}

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

pub fn execute_swap(deps: DepsMut, self_address: String, info: MessageInfo, pool_id: u64, token_out_denom: String, token_out_min_amount: String, to: Addr) -> Result<Response, ContractError> {
    let funds = info.funds.clone().pop().unwrap();
    let coin = Osmo_Coin {
        denom: funds.denom,
        amount: String::from("230000")
        //amount: funds.amount.to_string()
    };

    let mut osmo_routes: Vec<Osmo_SwapAmountInRoute> = Vec::new();
    osmo_routes.push(Osmo_SwapAmountInRoute {
        pool_id,
        token_out_denom: token_out_denom.clone(),
    });

    let msg = MsgSwapExactAmountIn {
        sender: self_address,
        routes: osmo_routes,
        token_in: Option::from(coin),
        token_out_min_amount,
    };

    let data = SwapMsgReplyState{denom_out: token_out_denom, to};

    SWAP_TO_REPLY_STATES.save(
        deps.storage,
        SWAP_REPLY_ID,
        &data
    )?;

    let msg = msg.to_msg()?;

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(msg, SWAP_REPLY_ID))
        .add_attribute("method", "execute_swap")
    )

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

pub const SWAP_EVENT: &str = "token_swapped";
pub const SWAP_ATTR: &str = "tokens_out";

pub fn handle_swap_reply(
    _deps: DepsMut,
    msg: Reply,
    send_to: SwapMsgReplyState,
) -> Result<Response, ContractError> {

    match msg.result {
        SubMsgResult::Ok(tx) => {
            let gamm_res = parse_gamm_result::<MsgSwapExactAmountInResponse>(tx);
                
            match gamm_res {
                Ok(amount_out) => {
                    let mut new_coin = Vec::new();
                    new_coin.push(Coin::new(amount_out, send_to.denom_out));

                    let bank_msg = BankMsg::Send {
                        to_address: send_to.to.into_string(),
                        amount: new_coin,
                    };
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


pub fn parse_gamm_result<M: GammResult + cw_osmo_proto::Message + std::default::Default>(
    msg: SubMsgResponse
) -> Result<u128, ContractError> {

    let data = msg.data.ok_or(ContractError::NoReplyData {})?;
    let response: M = proto_decode(data.as_slice())?;
    let amount = response
        .amount()
        .parse::<u128>()
        .map_err(|_| ContractError::InvalidAmountValue {})?;

    Ok(amount)
}
