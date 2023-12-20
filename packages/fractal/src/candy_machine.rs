use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{ Asset};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemainingToken {
    pub prefix: String,
    pub count: u64
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetConfig {
        is_open: bool,
        enable_whitelist: bool,
        round: u64
    },
    SetNftAddress{
        addr: String
    },
    Mint {},
    UpdateWhitelist {
        register_addr: String,
        count: u64,
        round: u64,
        is_delist: Option<bool>,
    },
    SetRandomSeed {
        seeds: Vec<RemainingToken>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    WhitelistSingle {
        addr: String
    },
    WhitelistAddress {
        addr: String,
        round: u64
    },
    Seed {

    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub token_addr: String,
    pub mint_asset: Asset,
    pub round: u64,
    pub protocol_fee: Decimal,
    pub creator: String,
    pub collector: String,
    pub total_token_count: Uint128,
    pub enable_whitelist: bool,
    pub total_supply: Uint128,
    pub is_open: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
