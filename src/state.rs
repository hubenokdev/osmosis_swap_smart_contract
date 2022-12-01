use cosmwasm_std::{Addr, Storage, Uint128};
use cw_storage_plus::Map;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

static CONFIG_KEY: &[u8] = b"config";

pub const BOT_KEY: &str = "bot_role";
pub const BOT_ROLES: Map<Addr, bool> = Map::new(BOT_KEY);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub pending_platform_fee: Uint128,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}


// Mapping between connections and the counter on that connection.
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