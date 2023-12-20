use cosmwasm_std::{Addr, Storage, Uint128, Decimal};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{Asset};
use fractal::candy_machine::{RemainingToken};

use crate::error::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub creator: Addr,
    pub token_addr: Addr,
    pub protocol_fee: Decimal,
    pub mint_asset: Asset,
    pub collector: Addr,
    pub enable_whitelist: bool,
    pub total_supply: Uint128,
    pub total_token_count: Uint128,
    pub is_open: bool,
    pub round: u64
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELISTS: Map<(&Addr, u64), Whitelist> = Map::new("whitelist");
pub const TOKEN_IDS: Item<Vec<RemainingToken>> = Item::new("token_ids");
pub const LAST_MINTER: Item<Addr> = Item::new("last_minter");
pub const LAST_TOKEN_ID: Item<String> = Item::new("last_token_id");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Whitelist {
    pub addr: Addr,
    pub round: u64,
    pub count: u64
}

/// Check whether the validator is whitelisted.
pub fn is_valid_whitelist(
    storage: &dyn Storage,
    address: Addr,
    round: u64
) -> Result<bool, ContractError> {
    let config = CONFIG.load(storage)?;
    match WHITELISTS.load(storage, (&address, round)) {
        Ok(whitelist) => {
            if whitelist.count > 0 && config.round == whitelist.round {
                return Ok(true);
            }
            return Ok(false);
        },
        _ => Ok(false),
    }
}
