use thiserror::Error;

type FogValue = fog_pack::types::Value;
type FogValueRef<'a> = fog_pack::types::ValueRef<'a>;
type JsonValue = serde_json::Value;
type JsonNumber = serde_json::Number;
type JsonMap = serde_json::Map<String, JsonValue>;
const FOG_PREFIX: &str = "$fog-";

mod enc;
mod dec;
mod doc;
mod entry;

use std::collections::BTreeMap;

pub use enc::{fog_to_json, fogref_to_json};
pub use dec::{json_to_fog, DecodeError};
pub use doc::{doc_to_json, json_to_doc};
pub use entry::{entry_to_json, json_to_entry};

/// An error that occurred while converting from JSON to a fog-pack object, like a Document or 
/// Entry.
#[derive(Clone, Debug, Error)]
pub enum ObjectError {
    /// Data conversion failed for a particular key-value pair in the object
    #[error("Data conversion failed for key {key}")]
    Decode {
        key: &'static str,
        #[source]
        src: DecodeError,
    },
    /// Expected a different data type
    #[error("Wrong data type for key \"{0}\"")]
    WrongDataType(&'static str),
    /// The root JSON value wasn't an Object as expected
    #[error("Expected a root Object for Doc/Entry/Query conversion")]
    NotAnObject,
    /// The object contained an unexpected key-value pair
    #[error("Unrecognized key (\"{0}\") while parsing fog-pack Object")]
    UnrecognizedKey(String),
    /// Missing one of the required key-value pairs for this fog-pack object
    #[error("Missing required key \"{0}\" for root object")]
    MissingKey(&'static str),
    /// Couldn't form the final result for some fog-pack specific reason
    #[error("Failed to form the fog-pack result")]
    FogPack(#[from] fog_pack::error::Error),
    /// Missing a key vault, which is needed to sign fog-pack objects
    #[error("Signing was requested, but no key vault was provided to look up the signing key")]
    NoVault,
    /// The key vault was missing the IdentityKey needed for signing the object
    #[error("Missing Identity Key for signing: {0}")]
    MissingIdentityKey(Box<fog_pack::types::Identity>),
}


#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {
    }
}
