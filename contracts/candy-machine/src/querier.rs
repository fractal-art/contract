use cosmwasm_std::{
  to_binary, Deps, QueryRequest, WasmQuery, Env, Uint128, MessageInfo, Decimal
};
use cw721::{Cw721QueryMsg, NumTokensResponse, TokensResponse};
use fractal::candy_machine::{ConfigResponse, RemainingToken};

use crate::error::ContractError;
use crate::state::{ CONFIG, Whitelist, WHITELISTS, TOKEN_IDS };

pub fn query_nft_total_supply(deps: Deps, token_addr: String) -> Result<u64, ContractError> {
  let total_supply_response: NumTokensResponse = 
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
      contract_addr: token_addr,
      msg: to_binary(&Cw721QueryMsg::NumTokens {})?
    }))?;
  Ok(total_supply_response.count)
}

pub fn only_owner(deps: Deps, info: MessageInfo) -> Result<bool, ContractError> {
  let config = CONFIG.load(deps.storage)?;
  if config.owner != info.sender {
    return Err(ContractError::Unauthorized {})
  }
  Ok(true)
}

pub fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
  let config = CONFIG.load(deps.storage)?;
  let token_addr = config.token_addr.to_string();
  Ok(ConfigResponse {
      owner: config.owner.to_string(),
      token_addr,
      mint_asset: config.mint_asset,
      round: config.round,
      total_token_count: config.total_token_count,
      total_supply: config.total_supply,
      enable_whitelist: config.enable_whitelist,
      is_open: config.is_open,
      collector: config.collector.to_string(),
      creator: config.creator.to_string(),
      protocol_fee: config.protocol_fee
  })
}

pub fn query_whitelist_single(
  deps: Deps,
  addr: String,
) -> Result<Option<Whitelist>, ContractError> {
  let config = CONFIG.load(deps.storage)?;
  let addr_raw = deps.api.addr_validate(&addr)?;
  let whitelist =  WHITELISTS.may_load(deps.storage, (&addr_raw, config.round))?;
  Ok(whitelist)
}

pub fn query_whitelist_address(
  deps: Deps,
  addr: String,
  round: u64
) -> Result<Option<Whitelist>, ContractError> {
  let addr_raw = deps.api.addr_validate(&addr)?;
  let whitelist =  WHITELISTS.may_load(deps.storage, (&addr_raw, round))?;
  Ok(whitelist)
}

pub fn query_seed(
  deps: Deps
) -> Result<Vec<RemainingToken>, ContractError> {
  let seed = TOKEN_IDS.load(deps.storage)?;
  Ok(seed)
}

pub fn query_nft_token_ids(deps: Deps, env: Env, token_id: String) -> Result<TokensResponse, ContractError> {
  let config = CONFIG.load(deps.storage)?;
  let tokens_response =
      deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
          contract_addr: config.token_addr.to_string(),
          msg: to_binary(&Cw721QueryMsg::Tokens {
              owner: env.contract.address.to_string(),
              start_after: Some(token_id),
              limit: Some(30),
          })?,
      }))?;
  return Ok(tokens_response);
}

pub fn calculate_fee(percent: Decimal, amount: Uint128) -> Result<Uint128, ContractError> {
  let fee = amount * percent;
  Ok(fee)
}