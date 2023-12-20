use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{ Asset};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token_addr: String,
    pub creator: String,
    pub mint_asset: Asset,
    pub protocol_fee: Decimal,
    pub enable_whitelist: bool,
    pub collector: String,
    pub total_supply: Uint128,
    pub total_token_count: Uint128,
}
