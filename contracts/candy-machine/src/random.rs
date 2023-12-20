use cosmwasm_std::{Deps, Env};
use sha3::{Keccak256, Digest};
use std::convert::TryInto;

use crate::error::ContractError;

pub fn random(
  _deps: Deps, 
  env: Env,
  address: String,
  last_minter: String,
  token_id: String,
) -> Result<u128, ContractError> {
  let mut text:String = format!("{}", address).to_owned();
  text.push_str(&format!("_{}", last_minter));
  text.push_str(&format!("_{}", token_id));
  // env variable
  text.push_str(&format!("_{}", env.block.height));
  text.push_str(&format!("_{}", env.block.time));
  let id = match env.transaction {
    Some(t) => t.index,
    None => 0
  };
  text.push_str(&format!("_{}", id));
  // hash
  let mut hasher = Keccak256::new();
  hasher.update(text);
  let hash = hasher.finalize();
  let (int_bytes, _rest) = hash[..].split_at(std::mem::size_of::<u128>());
  Ok(u128::from_le_bytes(int_bytes.try_into().unwrap()))
}
