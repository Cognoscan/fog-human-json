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
//! When going from JSON to a Document or Entry, if there's a "signer" specified, an intermediate 
//! struct will be provided that must be signed by a 
//! [`IdentityKey`][fog_crypto::identity::IdentityKey] that matches the signer.
//!
//! As an example, let's take a struct that looks the one below, put it into a document, and look 
//! at the resulting JSON:
//!
//! ```
//! # use std::collections::BTreeMap;
//! # use fog_crypto::identity::IdentityKey;
//! # use serde::{Serialize, Deserialize};
//! use fog_pack::{types::*, schema::NoSchema};
//! use fog_human_json::*;
//!
//! #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//! struct Test {
//!     boolean: bool,
//!     int: i64,
//!     float32: f32,
//!     float64: f64,
//!     #[serde(with = "serde_bytes")]
//!     bin: Vec<u8>,
//!     string: String,
//!     array: Vec<u32>,
//!     map: BTreeMap<String, u32>,
//!     time: Timestamp,
//!     hash: Hash,
//!     id: Identity,
//! }
//! 
//! // ... Fill it up with some data ...
//! # let bin = vec![0u8,1,2,3,4];
//! # let hash = Hash::new(&bin);
//! # let mut map = BTreeMap::new();
//! # map.insert(String::from("a"), 1u32);
//! # map.insert(String::from("b"), 2u32);
//! # map.insert(String::from("c"), 3u32);
//! # let time = Timestamp::now().unwrap();
//! #
//! # let mut rng = rand::thread_rng();
//! # let id_key = IdentityKey::new_temp(&mut rng);
//! # let id = id_key.id().clone();
//! #
//! # let test = Test {
//! #     boolean: false,
//! #     int: -12345,
//! #     float32: 0.0f32,
//! #     float64: 0.0f64,
//! #     bin,
//! #     string: "hello".into(),
//! #     array: vec![0,1,2,3,4],
//! #     map,
//! #     time,
//! #     hash,
//! #     id,
//! # };
//! #
//! let test: Test = test;
//!
//! let doc = fog_pack::document::NewDocument::new(None, &test).unwrap();
//! let doc = NoSchema::validate_new_doc(doc).unwrap();
//!
//! let json_val = doc_to_json(&doc);
//! let json_raw = serde_json::to_string_pretty(&json_val).expect("JSON Value to raw string");
//! ```
//! 
//! The resulting JSON could look something like:
//!
//! ```text
//! {
//!   "data": {
//!     "array": [ 0, 1, 2, 3, 4 ],
//!     "bin": "$fog-Bin:AAECAwQ",
//!     "boolean": false,
//!     "float32": "$fog-F32:0.0",
//!     "float64": 0.0,
//!     "hash": "$fog-Hash:R7KEBd4fxeYgDtoivjDUK97HwEcL7k7hm3qjPhZFEhzL",
//!     "id": "$fog-Identity:T4MvqAy6RVR2J8efJzgQW9xN9Z8avJBFEmefuSnBMWQP",
//!     "int": -12345,
//!     "lock": "$fog-LockId:ME7DmA9ADSYE6sq8SRvQ2ncd1kosQZoZqG7XiCFX55Uz",
//!     "map": {
//!       "a": 1,
//!       "b": 2,
//!       "c": 3
//!     },
//!     "stream_id": "$fog-StreamId:U4sLqPrtAgzKbUVnr47PcgPT2Rq3D9kBqrvhZ9NvCTvq",
//!     "string": "hello",
//!     "time": "$fog-Time:2023-07-12T17:33:13.454466675Z"
//!   }
//! }
//! ```
//!

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
mod query;

use std::collections::BTreeMap;

pub use enc::{fog_to_json, fogref_to_json};
pub use dec::{json_to_fog, DecodeError};
pub use doc::*;
pub use entry::*;
pub use query::*;

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
    /// The provided key was incorrect
    #[error("Incorrect Identity Key for signing, needed {0}")]
    IncorrectIdentityKey(Box<fog_pack::types::Identity>),
}


#[cfg(test)]
mod tests {
    use super::*;
    use fog_crypto::{identity::IdentityKey, stream::StreamKey, lock::LockKey};
    use fog_pack::{types::*, schema::NoSchema};
    use serde::{Deserialize, Serialize};

    #[test]
    fn back_and_forth() {

        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        struct Test {
            boolean: bool,
            int: i64,
            float32: f32,
            float64: f64,
            #[serde(with = "serde_bytes")]
            bin: Vec<u8>,
            string: String,
            array: Vec<u32>,
            map: BTreeMap<String, u32>,
            time: Timestamp,
            hash: Hash,
            id: Identity,
            stream_id: StreamId,
            lock: LockId,
            databox: DataLockbox
        }

        
        let bin = vec![0u8,1,2,3,4];
        let hash = Hash::new(&bin);
        let mut map = BTreeMap::new();
        map.insert(String::from("a"), 1u32);
        map.insert(String::from("b"), 2u32);
        map.insert(String::from("c"), 3u32);
        let time = Timestamp::now().unwrap();

        let mut rng = rand::thread_rng();
        let id_key = IdentityKey::new_temp(&mut rng);
        let id = id_key.id().clone();
        let stream_key = StreamKey::new_temp(&mut rng);
        let stream_id = stream_key.id().clone();
        let lock_key = LockKey::new_temp(&mut rng);
        let lock = lock_key.id().clone();
        let databox = lock.encrypt_data(&mut rng, &[0u8, 1, 2, 3]);

        let test = Test {
            boolean: false,
            int: -12345,
            float32: 0.0f32,
            float64: 0.0f64,
            bin,
            string: "hello".into(),
            array: vec![0,1,2,3,4],
            map,
            time,
            hash,
            id,
            stream_id,
            lock,
            databox,
        };

        let doc = fog_pack::document::NewDocument::new(None, &test).unwrap();
        let doc = NoSchema::validate_new_doc(doc).unwrap();

        let json_val = doc_to_json(&doc);
        let json_raw = serde_json::to_string(&json_val).expect("JSON Value to raw string");
        let parsed_json: JsonValue = serde_json::from_str(&json_raw).expect("Parsed JSON");
        assert_eq!( json_val, parsed_json);

        let parsed_doc = json_to_doc(&parsed_json).expect("JSON to document");
        let MaybeDocument::NewDocument(parsed_doc) = parsed_doc else {
            panic!("Document shouldn't have needed signing")
        };
        assert_eq!(parsed_doc.hash(), doc.hash());

        let parsed_doc = NoSchema::validate_new_doc(parsed_doc).expect("Validated document");
        let roundtrip_test: Test = parsed_doc.deserialize().expect("Deserialized correctly");

        assert!(roundtrip_test == test);
    }
}
