use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128, Uint64, Coin};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetAdmin {
        new_admin: Addr,
    },
    SetBotRole {
        new_bot: Addr,
        enabled: bool
    },
    WithdrawFee {
        // release some coins - if quantity is None, release all coins in balance
        to: Addr,
        amount: Uint128,
    },
    BuyToken { 
        osmo_amount: Uint128
        , pool_id: u64
        , denom_token: String
        , token_amount_per_native: Uint128
        , slippage_bips: Uint128
        , recipient: Addr
        , platform_fee_bips: Uint128
        , gas_estimate: Uint128
        , deadline: Uint64
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcExecuteMsg {
    IbcContractHop {
        commands: Vec<ExecuteMsg>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns a human-readable representation of the arbiter.
    GetInfos {
        token: String,
    },    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllInfos {
    pub admin: Addr,
    pub pending_platform_fee: Uint128,
    pub blocktime: u64,
    pub token_balance: Uint128,
    pub token_balances: Vec<Coin>,
    pub contract_address: Addr,
    //pub all_tokens: Vec<Coin>
}