use cosmwasm_std::{
  to_binary, DepsMut, MessageInfo, CosmosMsg, WasmMsg, Env, Uint128, Response, Deps
};
use cw721::{ Cw721ExecuteMsg };
use terraswap::asset::{Asset};
use std::convert::TryFrom;
use fractal::candy_machine::{RemainingToken};

use crate::error::ContractError;
use crate::state::{ CONFIG, TOKEN_IDS, WHITELISTS, Whitelist, is_valid_whitelist, LAST_MINTER, LAST_TOKEN_ID };
use crate::querier::{ only_owner, query_nft_token_ids, calculate_fee };
use crate::random::random;

pub fn set_config(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  is_open: bool,
  enable_whitelist: bool,
  round: u64
) -> Result<Response, ContractError> {
  only_owner(deps.as_ref(), info)?;

  let mut config = CONFIG.load(deps.storage)?;
  config.is_open = is_open;
  config.enable_whitelist = enable_whitelist;
  config.round = round;
  CONFIG.save(deps.storage, &config)?;

  Ok(Response::default())
}

pub fn set_nft_address(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  addr: String
) -> Result<Response, ContractError> {
  only_owner(deps.as_ref(), info)?;

  let mut config = CONFIG.load(deps.storage)?;
  config.token_addr = deps.api.addr_validate(&addr)?;
  CONFIG.save(deps.storage, &config)?;

  Ok(Response::default())
}

pub fn set_random_seeds(
  deps: DepsMut, 
  info: MessageInfo, 
  seeds: Vec<RemainingToken>
) -> Result<Response, ContractError> {
  only_owner(deps.as_ref(), info)?;

  TOKEN_IDS.save(deps.storage, &seeds)?;
  Ok(Response::new())
}

pub fn update_whitelist(
  deps: DepsMut,
  info: MessageInfo,
  register_addr: String,
  count: u64,
  round: u64,
  is_delist: Option<bool>,
) -> Result<Response, ContractError> {
  only_owner(deps.as_ref(), info)?;

  let is_delist = is_delist.unwrap_or(false);
  let register_addr = deps.api.addr_validate(&register_addr)?;

  let whitelist =  WHITELISTS.may_load(deps.storage, (&register_addr, round))?;

  if let Some(_whitelist) = whitelist {
      //Do nothing
  } else {
      if is_delist{
          return Err(ContractError::InvalidAddesss("Address doesn't exist".to_string()));
      }
  }

  if is_delist {
      WHITELISTS.remove(deps.storage, (&register_addr, round));
  } else {
      let whitelist = Whitelist {
          addr: register_addr.clone(),
          count: count,
          round: round
      };
      WHITELISTS.save(deps.storage, (&register_addr, round), &whitelist)?;
  }
  Ok(Response::default())
}

pub fn mint(
  mut deps: DepsMut, 
  env: Env, 
  info: MessageInfo
) -> Result<Response, ContractError> {
  let mut config = CONFIG.load(deps.storage)?;
  let sender = info.sender.clone();

  if config.enable_whitelist{
      let is_valid = is_valid_whitelist(
          deps.storage,
          sender.clone(),
          config.round
      )?;
      if !is_valid {
          return Err(ContractError::InvalidAddesss("not in whitelist".to_string()));
      }
  }

  //Check for minting condition
  if !config.is_open {
      return Err(ContractError::InvalidState("Candy Machine is not yet open".to_string()));
  }

  if config.mint_asset.amount != Uint128::zero() {
      config.mint_asset.assert_sent_native_token_balance(&info)?;
  }

  let remaining_token_ids = TOKEN_IDS.load(deps.storage)?.clone();
  let last_minter = LAST_MINTER.load(deps.storage)?;
  let last_token_id = LAST_TOKEN_ID.load(deps.storage)?;
  let prefix_rand = range_random(
    deps.as_ref(), 
    env.clone(),
    info.sender.to_string(),
    last_minter.to_string(),
    last_token_id.clone(),
    u128::try_from(remaining_token_ids.len()).unwrap() - 1)?;
  if  prefix_rand > remaining_token_ids.len(){
      let prefix_rand_err = String::from("Wrong Index From Prefix ") + &prefix_rand.to_string();
      return Err(ContractError::InvalidRandom(prefix_rand_err));
  }
  let random_token_id = &remaining_token_ids[prefix_rand].clone();
  let prefix_token_id = &random_token_id.prefix;

  let mut token_ids = query_nft_token_ids(deps.as_ref(), env.clone(), prefix_token_id.to_string())?;
  if token_ids.tokens.len() <= 0 {
      token_ids = query_nft_token_ids(deps.as_ref(), env.clone(), "".to_string())?;
  }

  let token_ids_rand = range_random(
    deps.as_ref(), 
    env.clone(),
    info.sender.to_string(),
    last_minter.to_string(),
    last_token_id.clone(),
    u128::try_from(token_ids.tokens.len()).unwrap() - 1)?;

  let token_id = &token_ids.tokens[token_ids_rand];
  let token_id_prefix = token_id.chars().nth(0).unwrap();
  decrease_count(deps.branch(), token_id_prefix.to_string())?;
  let mut messages: Vec<CosmosMsg> = vec![];

  if config.mint_asset.amount != Uint128::zero() {
      let mint_fee_amount = config.mint_asset.amount;
      let protocol_fee = calculate_fee(config.protocol_fee, mint_fee_amount)?;
      let seller_amount = mint_fee_amount - protocol_fee;
      let seller_mint_fee = Asset {
          amount: seller_amount,
          info: config.mint_asset.info.clone(),
      };
      messages.push(seller_mint_fee.into_msg(config.creator.clone())?);

      if protocol_fee > Uint128::from(0u128) {
          let protocol_mint_fee = Asset {
              amount: protocol_fee,
              info: config.mint_asset.info.clone(),
          };
          messages.push(protocol_mint_fee.into_msg(config.collector.clone())?);
      }
  }

  let transfer_nft = CosmosMsg::Wasm(WasmMsg::Execute {
      contract_addr: config.token_addr.to_string(),
      msg: to_binary(&Cw721ExecuteMsg::TransferNft {
          token_id: token_id.clone(),
          recipient: info.sender.to_string(),
      })?,
      funds: vec![],
  });
  messages.push(transfer_nft);

  config.total_token_count -= Uint128::from(1u128);
  CONFIG.save(deps.storage, &config)?;

  LAST_MINTER.save(deps.storage, &info.sender)?;
  LAST_TOKEN_ID.save(deps.storage, &token_id)?;

  if config.enable_whitelist{
      decrease_whitelist_count(deps, info, config.round)?;
  }

  Ok(Response::new()
      .add_messages(messages)
      .add_attribute("action", "mint")
      .add_attribute("id", token_id.clone()))
}

fn range_random(
  deps: Deps,
  env: Env,
  address: String,
  last_minter: String,
  last_token_id: String,
  range: u128
) -> Result<usize, ContractError> {
  if range == 0{
      return Ok(0);
  }
  let entropy = random(
    deps, 
    env.clone(),
    address,
    last_minter,
    last_token_id)?;
  let random_index = entropy % range;
  let idx = usize::try_from(random_index).unwrap();
  return Ok(idx);
}

fn decrease_count(
  deps: DepsMut,
  search_token_id: String
) -> Result<String, ContractError> {
  let mut remaining_token_ids = TOKEN_IDS.load(deps.storage)?.clone();
  let index = remaining_token_ids.iter().position(|id| id.prefix == search_token_id).unwrap_or(usize::MAX);
  if index == usize::MAX{
      let err = String::from("Prefix Find Failed") + &search_token_id;
      return Err(ContractError::InvalidState(err));
  }
  let current_index_count = remaining_token_ids[index].count - 1;
  if current_index_count == 0 {
      remaining_token_ids.remove(index);
  }else {
      remaining_token_ids[index].count = current_index_count;
  }

  TOKEN_IDS.save(deps.storage, &remaining_token_ids)?;
  Ok("Cool!".to_string())
}

pub fn decrease_whitelist_count(
  deps: DepsMut,
  info: MessageInfo,
  round: u64
) -> Result<Response, ContractError> {
  let whitelist =  WHITELISTS.may_load(deps.storage, (&info.sender, round.clone()))?;
  if let Some(mut whitelist) = whitelist {
      whitelist.count -= 1;
      WHITELISTS.save(deps.storage, (&info.sender, round.clone()), &whitelist)?;
      Ok(Response::default())
  }else{
      return Err(ContractError::InvalidAddesss("Address doesn't exist".to_string()));
  }
}