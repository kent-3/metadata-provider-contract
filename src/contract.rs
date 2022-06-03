use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HandleResult, 
    InitResponse, InitResult, Querier, QueryResult, StdError, StdResult, Storage, HumanAddr,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, HandleAnswer, QueryAnswer, ResponseStatus::Success};
use crate::state::{
    json_may_load, json_save, load, may_load, remove, save,
    Config,
    BLOCK_KEY, CONFIG_KEY, CREATOR_KEY, MY_ADDRESS_KEY, PRNG_SEED_KEY, PREFIX_VIEW_KEY,
    PREFIX_PUB_META, PREFIX_PRIV_META
};

use secret_toolkit::snip721::{Metadata, Extension, Trait, MediaFile, Authentication};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use secret_toolkit::viewing_key::{ViewingKey, VIEWING_KEY_SIZE, ViewingKeyStore};

/// pad handle responses and log attributes to blocks of 256 bytes to prevent leaking info based on
/// response size
pub const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    let config = Config {
        name: msg.name,
        symbol: msg.symbol,
        admin: deps.api.canonical_address(&env.message.sender)?,
        token_address: deps.api.canonical_address(&msg.token_address)?,
        token_code_hash: msg.token_code_hash,
    };

    save(&mut deps.storage, CONFIG_KEY, &config)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    save(&mut deps.storage, BLOCK_KEY, &env.block)?;
    let mut config: Config = load(&deps.storage, CONFIG_KEY)?;
    
    let response = match msg {
        HandleMsg::SetMetadata { 
            token_id, 
            public_metadata, 
            private_metadata, 
            ..
        } => handle_set_metadata(
            deps,
            env,
            &config,
            &token_id,
            public_metadata,
            private_metadata,
        ),
        HandleMsg::CreateViewingKey { entropy, .. } => handle_create_key(
            deps,
            env,
            &config,
            &entropy,
        ),
        HandleMsg::SetViewingKey { key, .. } => handle_set_key(
            deps,
            env,
            &config,
            key,
        ),
        HandleMsg::ChangeAdmin { address, .. } => handle_change_admin(
            deps,
            env,
            &mut config,
            &address,
        ),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

pub fn handle_set_metadata<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &Config,
    token_id: &str,
    public_metadata: Option<Metadata>,
    private_metadata: Option<Metadata>,
) -> HandleResult {

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetMetadata {
            status: Success,
        })?),
    })
}

pub fn handle_create_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &Config,
    entropy: &str,
) -> HandleResult {
    let key = ViewingKey::create(&mut deps.storage, &env, &env.message.sender, entropy.as_ref());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ViewingKey {
            key,
        })?),
    })
}

pub fn handle_set_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &Config,
    key: String,
) -> HandleResult {
    ViewingKey::set(&mut deps.storage, &env.message.sender, key.as_str());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ViewingKey {
            key,
        })?),
    })
}

pub fn handle_change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &mut Config,
    address: &HumanAddr,
) -> HandleResult {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    if config.admin != sender_raw {
        return Err(StdError::generic_err(
            "This is an admin command and can only be run from the admin address",
        ));
    }
    let new_admin = deps.api.canonical_address(address)?;
    if new_admin != config.admin {
        config.admin = new_admin;
        save(&mut deps.storage, CONFIG_KEY, &config)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin {
            status: Success,
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let response = match msg {
        QueryMsg::NftInfo { token_id } => {
            query_nft_info(deps, &token_id)
        }
        QueryMsg::PrivateMetadata { token_id, viewer } => {
            query_private_metadata(deps, &token_id)
        }
    };
    pad_query_result(response, BLOCK_SIZE)
}

fn query_nft_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: &str,
) -> QueryResult {
    
    let meta_store = ReadonlyPrefixedStorage::new(PREFIX_PUB_META, &deps.storage);
    let meta: Metadata = may_load(&meta_store, token_id.as_ref())?.unwrap_or_default();

    to_binary(&QueryAnswer::NftInfo {
        token_uri: meta.token_uri,
        extension: meta.extension,
    })
}

fn query_private_metadata<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: &str,
) -> QueryResult {
    let meta_store = ReadonlyPrefixedStorage::new(PREFIX_PRIV_META, &deps.storage);
    let meta: Metadata = may_load(&meta_store, token_id.as_ref())?.unwrap_or_default();

    to_binary(&QueryAnswer::PrivateMetadata {
        token_uri: meta.token_uri,
        extension: meta.extension,
    })
}
