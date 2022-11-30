#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point,  DepsMut, Env, MessageInfo, Response, BankMsg, StdError, IbcMsg, Coin, SubMsg, Addr
    , SubMsgResult, SubMsgResponse, Reply, Uint128, Event, Attribute};
use cw2::set_contract_version;
use cw_osmo_proto::osmosis::gamm::v1beta1::{ MsgSwapExactAmountIn, MsgSwapExactAmountInResponse, SwapAmountInRoute as Osmo_SwapAmountInRoute };
use cw_osmo_proto::cosmos::base::v1beta1::{ Coin as Osmo_Coin };
use cw_osmo_proto::proto_ext::{MessageExt};
use cw_osmo_proto::proto_ext::proto_decode;
// use cw721_base::{
//     msg::ExecuteMsg as Cw721ExecuteMsg, Extension,
//     MintMsg,
// };
// use std::convert::TryInto;
// use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{SWAP_TO_REPLY_STATES, AmountResultAck, SwapMsgReplyState};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:blazarbit-protocol";
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

pub fn find_event_type(events: Vec<Event>, key: &str) -> Option<Event> {
    for ev in events {
        if ev.ty.eq(&key) {
            return Some(ev);
        }
    }

    None
}

pub fn find_attributes(attributes: Vec<Attribute>, key: &str) -> Vec<String> {
    let mut values = vec![];
    for attr in attributes {
        if attr.key.eq(&key) {
            values.push(attr.value)
        }
    }

    values
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
// impl GammResult for MsgJoinSwapExternAmountInResponse {
//     fn amount(&self) -> &String {
//         &self.share_out_amount
//     }
// }

// impl GammResult for MsgExitSwapShareAmountInResponse {
//     fn amount(&self) -> &String {
//         &self.token_out_amount
//     }
// }

// pub fn find_event_type(events: Vec<Event>, key: &str) -> Option<Event> {
//     for ev in events {
//         if ev.ty.eq(&key) {
//             return Some(ev);
//         }
//     }

//     None
// }

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
        ExecuteMsg::IbcTransfer { channel_id, address } => execute_ibc_transfer(deps, _env, info, channel_id, address),
        ExecuteMsg::Swap { pool_id, token_out_denom, token_out_min_amount, to } => execute_swap(deps, _env.contract.address.into(), info, pool_id, token_out_denom, token_out_min_amount, to),
        //ExecuteMsg::Swap { pool_id, token_out_denom, token_out_min_amount } => execute_swap(info.sender.clone().into(), info, pool_id, token_out_denom, token_out_min_amount),
        // ExecuteMsg::PurchaseNFT { owner, contract_addr, token_id, token_uri } => purchase_nft(deps, _env, info, contract_addr, token_id, token_uri, owner),
        // ExecuteMsg::ContractHop { contract_addr, commands } => contract_hop(deps, info, contract_addr, commands),
        // ExecuteMsg::IbcContractHop { channel, commands } => execute_ibc_contract_hop(_env, channel, commands),
    }
}

// fn execute_ibc_contract_hop(env: Env, channel: String, commands: Vec<ExecuteMsg>) -> Result<Response, ContractError> {
//     Ok(Response::new()
//         .add_attribute("method", "execute_ibc_contract_hop")
//         .add_attribute("channel", channel.clone())
//         .add_message(IbcMsg::SendPacket {
//             channel_id: channel,
//             data: to_binary(&IbcExecuteMsg::IbcContractHop { commands })?,
//             timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300)),
//         }))
// }

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

pub fn execute_ibc_transfer(_deps: DepsMut, env: Env, mut info: MessageInfo, channel_id: String, addr: String) -> Result<Response, ContractError> {
    // require some funds
    let amount = match info.funds.pop() {
        Some(coin) => coin,
        None => {
            return Err(ContractError::Std(StdError::generic_err(
                "you must send the coins you wish to ibc transfer",
            )))
        }
    };

    // construct a packet to send
    let msg = IbcMsg::Transfer {
        channel_id,
        to_address: addr,
        amount,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_ibc_transfer"))
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

    // Ok(Response::new()
    // .add_message(msg)
    // .add_attribute("action", "execute_swap"))

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(msg, SWAP_REPLY_ID))
        .add_attribute("method", "execute_swap")
    )

    // Ok(Response::new()
    //     .add_attribute("method", "execute_swap")
    //     .add_submessage(SubMsg::reply_on_success(msg, SWAP_REPLY_ID)))
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
        // Ok(Response::new()
        //     .add_attribute("Default", "reply"))
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
        // return Ok(Response::new()
        //     .add_attribute("1swap", "1reply"));
    match msg.result {
        SubMsgResult::Ok(tx) => {
            let gamm_res = parse_gamm_result::<MsgSwapExactAmountInResponse>(tx, SWAP_EVENT, SWAP_ATTR);
                
            match gamm_res {
                Ok(amount_out) => {
                    // return Ok(Response::new()
                    //     .add_attribute("---parse_gamm_result ok", ack.amount));
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
                    //Ok(Response::new().set_data(ack_fail(err.to_string())))
                }
            }
        }
        SubMsgResult::Err(err) => {
            Err(ContractError::FailedSwap {
                reason: err,
            })
        }
    }

    // if let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result {        
    //     //let res: MsgSwapExactAmountInResponse = b.try_into().map_err(ContractError::Std)?;
    //     let result = parse_gamm_result::<MsgSwapExactAmountInResponse>(msg, SWAP_EVENT, SWAP_ATTR);
        
    //     // let amount = Uint128::from_str(&res.token_out_amount)?;

    //     // let send_denom = &swap_msg_reply_state
    //     //     .swap_msg
    //     //     .routes
    //     //     .last()
    //     //     .unwrap()
    //     //     .token_out_denom;

    //     let bank_msg = BankMsg::Send {
    //         to_address: swap_msg_reply_state.to.into_string(),
    //         amount: coins(amount.u128(), swap_msg_reply_state.denom_out),
    //     };

    //     return Ok(Response::new()
    //         .add_message(bank_msg)
    //         .add_attribute("token_out_amount", amount));
    // }

    // Err(ContractError::FailedSwap {
    //     reason: msg.result.unwrap_err(),
    // })
}

// pub fn proto_decode<M: prost::Message + std::default::Default>(data: &[u8]) -> StdResult<M> {
//     prost::Message::decode(data).map_err(|_| StdError::generic_err("cannot decode proto"))
// }

pub fn parse_gamm_result<M: GammResult + cw_osmo_proto::Message + std::default::Default>(
    msg: SubMsgResponse,
    event: &str,
    attribute: &str,
) -> Result<u128, ContractError> {
    // let event = find_event_type(msg.events, event);
    // if event.is_none() {
    //     return Err(ContractError::GammResultNotFound {});
    // }

    // let values = find_attributes(event.unwrap().attributes, attribute);
    // if values.is_empty() {
    //     return Err(ContractError::GammResultNotFound {});
    // }

    // let token_out_str = values.last().unwrap();
    // let token_out = parse_coin(token_out_str.as_str())?;

    let data = msg.data.ok_or(ContractError::NoReplyData {})?;
    let response: M = proto_decode(data.as_slice())?;
    let amount = response
        .amount()
        .parse::<u128>()
        .map_err(|_| ContractError::InvalidAmountValue {})?;

    // let ack = AmountResultAck {
    //     amount: Uint128::from(amount),
    //     denom: token_out.denom,
    // };

    Ok(amount)
}

// todo: Purchase logic implemented via nft mint just for HackAtom explanation,
//  need to change it to the real NFT purchase on market
// pub fn purchase_nft(_deps: DepsMut, _env: Env, info: MessageInfo, contract_addr: String, token_id: String, token_uri: String, owner: String) -> Result<Response, ContractError> {
//     let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
//         token_id: token_id.to_string(),
//         owner: owner.clone(),
//         token_uri: token_uri.clone().into(),
//         extension: Option::None
//     });

//     let msg = WasmMsg::Execute {
//         contract_addr: contract_addr.clone(),
//         msg: to_binary(&mint_msg)?,
//         funds: info.funds.clone(),
//     };

//     Ok(Response::new()
//         .add_message(msg)
//         .add_attribute("action", "purchaseNft"))
// }

// pub fn contract_hop(deps: DepsMut, info: MessageInfo, contract_addr: String, mut commands: Vec<ExecuteMsg>) -> Result<Response, ContractError> {
//     match deps.api.addr_validate(contract_addr.as_str()).ok() {
//         None => return Err(ContractError::Unauthorized {}),
//         Some(addr) => CONTRACT_ADDRESS.save(deps.storage, &addr)?,
//     };

//     // todo: need to fix it:
//     //  Execute error: Broadcasting transaction failed with code 32 (codespace: sdk). Log: account sequence mismatch, expected 20, got 19: incorrect account sequence
//     // let funds: Vec<_> = info.funds.into_iter().map(|c| Coin{
//     //     denom: c.denom,
//     //     amount: c.amount / (Uint128::new(commands.len() as u128)),
//     // }).collect();
//     //
//     // let messages: Vec<_> = commands.into_iter().map(|cmd| {
//     //     WasmMsg::Execute {
//     //         contract_addr: contract_addr.clone(),
//     //         msg: to_binary(&cmd).unwrap(),
//     //         funds: funds.clone(),
//     //     }
//     // }).collect();

//     // let funds: Vec<_> = info.funds;
//     // let mut funds = info.funds;


//     let msgs = if let Some(command) = commands.pop() {
//         let msg = match command {
//             ExecuteMsg::Transfer { address } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::Transfer { address }).unwrap(),
//                     funds: info.funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::IbcTransfer { channel_id, address } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::IbcTransfer { channel_id, address }).unwrap(),
//                     funds: info.funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::Swap { pool_id, token_out_denom, token_out_min_amount } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::Swap {
//                         pool_id,
//                         token_out_denom,
//                         token_out_min_amount
//                     }).unwrap(),
//                     funds: info.funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::PurchaseNFT { owner, contract_addr, token_id, token_uri } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::PurchaseNFT {
//                         owner,
//                         contract_addr,
//                         token_id,
//                         token_uri
//                     }).unwrap(),
//                     funds: info.funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::ContractHop { contract_addr, commands } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::ContractHop { contract_addr, commands }).unwrap(),
//                     funds: info.funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::IbcContractHop { channel, commands } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::IbcContractHop { channel, commands }).unwrap(),
//                     funds: info.funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//         };
//         vec![msg]
//     } else { vec![] };

//     COMMANDS_STACK.save(deps.storage, &commands)?;

//     Ok(Response::new()
//         .add_attribute("method", "contract_hop")
//         .add_submessages(msgs))
// }

// // I don't know, can I delete this reply or not that's wy allow dead_code
// #[allow(dead_code)]
// #[entry_point]
// fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
//     match msg.id { 1 => {
//         hop_reply(deps, env, msg.result)
//     }
//         _ => return Err(ContractError::Unauthorized {})
//     }
// }

// pub fn hop_reply(deps: DepsMut, env: Env, msg: SubMsgResult) -> Result<Response, ContractError> {
//     msg.into_result().map_err(|err| StdError::generic_err(err))?;
//     let mut commands = COMMANDS_STACK.load(deps.storage)?;
//     let contract_addr = CONTRACT_ADDRESS.load(deps.storage)?.into_string();

//     let funds = deps.querier.query_all_balances(env.contract.address)?;
//     let msgs = if let Some(command) = commands.pop() {
//         let msg = match command {
//             ExecuteMsg::Transfer { address } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::Transfer { address }).unwrap(),
//                     funds: funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::IbcTransfer { channel_id, address } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::IbcTransfer { channel_id, address }).unwrap(),
//                     funds: funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::Swap { pool_id, token_out_denom, token_out_min_amount } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::Swap {
//                         pool_id,
//                         token_out_denom,
//                         token_out_min_amount
//                     }).unwrap(),
//                     funds: funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::PurchaseNFT { owner, contract_addr, token_id, token_uri } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::PurchaseNFT {
//                         owner,
//                         contract_addr,
//                         token_id,
//                         token_uri
//                     }).unwrap(),
//                     funds: funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::ContractHop { contract_addr, commands } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::ContractHop { contract_addr, commands }).unwrap(),
//                     funds: funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//             ExecuteMsg::IbcContractHop { channel, commands } => {
//                 let msg = WasmMsg::Execute {
//                     contract_addr: contract_addr.clone(),
//                     msg: to_binary(&ExecuteMsg::IbcContractHop { channel, commands }).unwrap(),
//                     funds: funds.clone(),
//                 };
//                 SubMsg::reply_on_success(msg, 1)
//             }
//         };
//         vec![msg]
//     } else { vec![] };

//     COMMANDS_STACK.save(deps.storage, &commands)?;
//     Ok(Response::new()
//         .add_attribute("method", "hop_reply").add_submessages(msgs))
// }
