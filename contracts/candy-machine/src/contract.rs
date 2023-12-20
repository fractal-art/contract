use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut,
    Env, MessageInfo, Response, StdResult,
};
use fractal::candy_machine::{ExecuteMsg, QueryMsg, MigrateMsg};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg};
use crate::state::{ Config, CONFIG, LAST_MINTER, LAST_TOKEN_ID };

use crate::querier::{query_seed, query_config, query_whitelist_single, query_whitelist_address };
use crate::execute::{ set_config, set_nft_address, set_random_seeds, update_whitelist, mint };

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let config = Config {
        owner: info.sender.clone(),
        token_addr: deps.api.addr_validate(&msg.token_addr)?,
        creator: deps.api.addr_validate(&msg.creator)?,
        mint_asset: msg.mint_asset,
        enable_whitelist: msg.enable_whitelist,
        protocol_fee: msg.protocol_fee,
        collector: deps.api.addr_validate(&msg.collector)?,
        total_supply: msg.total_supply,
        total_token_count: msg.total_token_count,
        round: 1,
        is_open: false,
    };
    CONFIG.save(deps.storage, &config)?;

    LAST_MINTER.save(deps.storage, &info.sender)?;
    LAST_TOKEN_ID.save(deps.storage, &"0".to_string())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetConfig { is_open, enable_whitelist, round } => set_config(deps, env, info, is_open, enable_whitelist, round),
        ExecuteMsg::SetNftAddress { addr } => set_nft_address(deps, env, info, addr),
        ExecuteMsg::Mint {} => mint(deps, env, info),
        ExecuteMsg::UpdateWhitelist {
            register_addr,
            count,
            round,
            is_delist,
        } => update_whitelist(deps, info, register_addr, count, round, is_delist),
        ExecuteMsg::SetRandomSeed {
            seeds
        } => set_random_seeds(deps, info, seeds)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::WhitelistSingle{ addr } =>{
            Ok(to_binary(&query_whitelist_single(deps, addr)?)?)
        },
        QueryMsg::WhitelistAddress { addr, round } => Ok(to_binary(&query_whitelist_address(deps, addr, round)?)?), 
        QueryMsg::Seed{  } =>{
            Ok(to_binary(&query_seed(deps)?)?)
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
