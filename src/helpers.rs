use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, StdResult, WasmMsg, SubMsgResponse, QuerierWrapper, Uint128, QueryRequest, BankQuery
    , StdError, BalanceResponse as NativeBalanceResponse, WasmQuery, Coin, AllBalanceResponse
};
use cw20::{Balance, Cw20ExecuteMsg, Denom, BalanceResponse as CW20BalanceResponse, Cw20QueryMsg};

use cw_osmo_proto::osmosis::gamm::v1beta1::{ MsgSwapExactAmountInResponse};
use cw_osmo_proto::proto_ext::proto_decode;

use crate::error::ContractError;

use crate::msg::{ExecuteMsg};
pub trait GammResult {
    fn amount(&self) -> &String;
}

impl GammResult for MsgSwapExactAmountInResponse {
    fn amount(&self) -> &String {
        &self.token_out_amount
    }
}

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
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

pub fn get_token_amount(
    querier: QuerierWrapper,
    denom: Denom,
    contract_addr: Addr
) -> Result<Uint128, StdError> {

    match denom.clone() {
        Denom::Native(native_str) => {
            let native_response: NativeBalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
                address: contract_addr.clone().into(),
                denom: native_str
            }))?;
            return Ok(native_response.amount.amount);
        },
        Denom::Cw20(cw20_address) => {
            let balance_response: CW20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: cw20_address.clone().into(),
                msg: to_binary(&Cw20QueryMsg::Balance {address: contract_addr.clone().into()})?,
            }))?;
            return Ok(balance_response.balance);
        }
    }
}

pub fn get_tokens_amounts(
    querier: QuerierWrapper,
    contract_addr: Addr
) -> Result<Vec<Coin>, StdError> {        
    let native_response: AllBalanceResponse = querier.query(
        &QueryRequest::Bank(BankQuery::AllBalances {
        address: contract_addr.clone().into(),
    }))?;
    return Ok(native_response.amount);        
}