//use cw_osmo_proto::osmosis::gamm::v1beta1::{ MsgSwapExactAmountIn, SwapAmountInRoute as Osmo_SwapAmountInRoute };
//use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountIn};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use cw_storage_plus::Map;
use crate::msg::ExecuteMsg;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub const COMMANDS_STACK: Item<Vec<ExecuteMsg>> = Item::new( "commands_stack");
pub const CONTRACT_ADDRESS: Item<Addr> = Item::new( "contract_address");
// Mapping between connections and the counter on that connection.
pub const CONNECTION_COUNTS: Map<String, u32> = Map::new("connection_counts");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq , JsonSchema)]
pub struct SwapMsgReplyState {
    pub denom_out: String,
    pub to: Addr,
}               

pub const SWAP_TO_REPLY_STATES: Map<u64, SwapMsgReplyState> = Map::new("swap_reply_states");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AmountResultAck {
    pub amount: Uint128,
    pub denom: String,
}