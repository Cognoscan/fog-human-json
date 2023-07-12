//! This crate provides functions to go back and forth between fog-pack and JSON, 
//! making it relatively easy for users to view pretty-printed fog-pack values and 
//! edit them with existing JSON tooling. A common complaint with binary data 
//! formats like fog-pack is that reading them is painful, and lowering that pain 
//! with JSON is exactly what this crate is for.
//! 
//! This is *not* a crate for turning regular JSON into fog-pack data. It uses a 
//! number of special string prefixes to encode fog-pack types in JSON, which can 
//! interfere with arbitrary JSON-to-fog conversions.
//! 
//! So, what does this actually do for conversion? Well, it takes each fog-pack type 
//! and either directly converts it to a corresponding JSON type, or it specially 
//! encodes it in a string that starts with `$fog-`. So a 32-bit floating point 
//! value could be specifically encoded as `$fog-F32: 1.23`. The full list of types 
//! is:
//! 
//! - Str: A regular string. This is just prepended so fog-pack strings that start 
//!   with `$fog-` won't get caught by the parser.
//! - Bin: Encodes the binary data as Base64 using the "standard" encoding (bonus 
//!   symbols of `+/`, no padding used, padding is accepted when parsing).
//! - F32Hex / F64Hex: Encodes a binary32/64 IEEE floating-point value in big-endian hex. 
//!   The fog-to-json process should only do this when writing out a NaN or 
//!   Infinity.
//! - F32 / F64 / Int: Prints a standard JSON Number, but includes the type 
//!   information. This done by telling the converter to do it specifically, by a 
//!   user adding type information, or by the converter for any F32 value (as 
//!   `serde_json` will always use F64 for floating-point).
//! - Time: Encodes the time as a RFC 3339 formatted string.
//! - Hash / Identity / StreamId / LockId: Encodes the corresponding primitive as a 
//!   base58 string (in the Bitcoin base58 style).
//! - DataLockbox / IdentityLockbox / StreamLockbox / LockLockbox: Encodes the 
//!   corresponding lockbox as Base64 data, just like with the "Bin" type.
//! 
//! That covers conversion between fog-pack Values and JSON values, but not 
//! Documents and Entries. Those are converted into JSON objects with the following 
//! key-value pairs:
//! 
//! - Documents:
//!   - "schema": If present, a `$fog-Hash:HASH` with the schema.
//!   - "signer": If present, a `$fog-Identity:IDENTITY` with the signer's 
//!     Identity. 
//!   - "compression": If not present, uses default compression. If present and 
//!     null, no compression is used. If set to a number between 0-255, uses that 
//!     as the compression level.
//!   - "data": The document content. Must be present.
//! - Entries:
//!   - "parent": Parent document's hash.
//!   - "key": Entry's string key.
//!   - "signer": If present, holds the signer's Identity.
//!   - "compression": If not present, uses default compression. If present and 
//!     null, no compression is used. If set to a number between 0 & 255, uses that 
//!     as the compression level.
//!   - "data": The entry content. Must be present.
//! 
//! When going from JSON to a Document or Entry, if there's a "signer" specified, it 
//! will attempt to pull a matching IdentityKey from a provided Vault and use that 
//! to reproduce the signature. If it can't, then the conversion will fail.

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
