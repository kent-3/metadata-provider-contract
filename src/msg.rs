use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::HumanAddr;
use secret_toolkit::snip721::{Extension, Metadata, ViewerInfo};
use secret_toolkit::permit::Permit;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    /// name of associated token contract
    pub name: String,
    /// associated token contract symbol
    pub symbol: String,
    /// admin address
    pub token_address: HumanAddr,
    /// associated token contract code hash
    pub token_code_hash: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// set the public and/or private metadata.  This can be called by either the token owner or
    /// a valid minter if they have been given this power by the appropriate config values
    SetMetadata {
        /// id of the token whose metadata should be updated
        token_id: String,
        /// the optional new public metadata
        public_metadata: Option<Metadata>,
        /// the optional new private metadata
        private_metadata: Option<Metadata>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// create a viewing key
    CreateViewingKey {
        /// entropy String used in random key generation
        entropy: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// set viewing key
    SetViewingKey {
        /// desired viewing key
        key: String,
        /// optional message length padding
        padding: Option<String>,
    },
    /// change address with administrative power
    ChangeAdmin {
        /// address with admin authority
        address: HumanAddr,
        /// optional message length padding
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum HandleAnswer {
    SetMetadata {
        status: ResponseStatus,
    },
    /// response from both setting and creating a viewing key
    ViewingKey {
        key: String,
    },
    ChangeAdmin {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// displays the public metadata of a token
    NftInfo { token_id: String },
    /// displays the private metadata if permitted to view it
    PrivateMetadata {
        token_id: String,
        /// optional address and key requesting to view the private metadata
        viewer: Option<ViewerInfo>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    NftInfo {
        token_uri: Option<String>,
        extension: Option<Extension>,
    },
    PrivateMetadata {
        token_uri: Option<String>,
        extension: Option<Extension>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}
