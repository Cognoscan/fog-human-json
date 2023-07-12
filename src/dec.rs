use super::*;

use thiserror::Error;

/// An error that occurred while converting from JSON to a fog-pack value.
#[derive(Clone, Debug, Error)]
pub enum DecodeError {
    /// Conversion failed on a specific value inside an array
    #[error("Couldn't convert array item {loc}")]
    Array {
        loc: usize,
        #[source]
        err: Box<DecodeError>
    },
    /// Conversion failed on a specific value inside an object
    #[error("Couldn't convert object value with key {key}")]
    Map {
        key: String,
        #[source]
        err: Box<DecodeError>
    },
    /// Base64 encoding for a fog-pack value was expected, but the encoding was invalid
    #[error("Invalid Base64")]
    Base64(base64::DecodeError),
    /// Hex encoding for a fog-pack value was expected, but the encoding was invalid
    #[error("Invalid hexadecimal")]
    Hex(#[from] hex::FromHexError),
    /// An unrecognized `$fog-TYPE:` was found
    #[error("Unrecognized fog-pack type \"{0}\"")]
    UnrecognizedType(String),
    /// The time format couldn't be parsed as RFC3339
    #[error("Invalid Time")]
    InvalidTime(#[from] chrono::format::ParseError),
    /// The floating-point value was invalid
    #[error("Invalid floating-point value")]
    InvalidFloat,
    /// The Integer value was invalid
    #[error("Invalid integer")]
    InvalidInteger,
    /// The `$fog-TYPE` was missing a colon between it and the type data
    #[error("Bad fogpack type (missing a colon at end of type)")]
    BadFogType,
    /// Base58 encoding for a fog-pack value was expected, but the encoding was invalid
    #[error("Invalid Base58")]
    InvalidBase58,
    /// A lockbox's data was invalid in some way
    #[error("Invalid Lockbox")]
    InvalidLockbox,
}

fn base64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    use base64::engine::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.decode(input).map_err(DecodeError::Base64)
}

/// Convert a JSON Value to a fog-pack value.
pub fn json_to_fog(val: &JsonValue) -> Result<FogValue, DecodeError> {
    Ok(match val {
        JsonValue::Null => FogValue::Null,
        JsonValue::Bool(b) => FogValue::Bool(*b),
        JsonValue::Array(a) => {
            let mut new_a = Vec::with_capacity(a.len());
            for (loc, v) in a.iter().enumerate() {
                new_a.push(json_to_fog(v).map_err(|e| DecodeError::Array { loc, err: Box::new(e) })?);
            }
            FogValue::Array(new_a)
        },
        JsonValue::Object(o) => {
            let mut map = std::collections::BTreeMap::new();
            for (k, v) in o.iter() {
                let new_v = json_to_fog(v).map_err(|e| DecodeError::Map { key: k.to_string(), err: Box::new(e) })?;
                map.insert(k.to_string(), new_v);
            }
            FogValue::Map(map)
        },
        JsonValue::Number(n) => {
            if let Some(v) = n.as_u64() {
                FogValue::Int(fog_pack::types::Integer::from(v))
            }
            else if let Some(v) = n.as_i64() {
                FogValue::Int(fog_pack::types::Integer::from(v))
            }
            else {
                FogValue::F64(n.as_f64().unwrap())
            }
        },
        JsonValue::String(s) => {
            if let Some(s) = s.strip_prefix(FOG_PREFIX) {
                let (ty, untrimmed_val) = s.split_once(':').ok_or(DecodeError::BadFogType)?;
                let val = untrimmed_val.trim();
                match ty {
                    "Str" => FogValue::Str(untrimmed_val.to_owned()),
                    "F32" => {
                        let f = val.parse::<f32>().map_err(|_| DecodeError::InvalidFloat)?;
                        FogValue::F32(f)
                    }
                    "F64" => {
                        let f = val.parse::<f64>().map_err(|_| DecodeError::InvalidFloat)?;
                        FogValue::F64(f)
                    }
                    "Int" => {
                        if val.starts_with('-') {
                            let v = val.parse::<i64>().map_err(|_| DecodeError::InvalidInteger)?;
                            FogValue::Int(fog_pack::types::Integer::from(v))
                        }
                        else {
                            let v = val.parse::<u64>().map_err(|_| DecodeError::InvalidInteger)?;
                            FogValue::Int(fog_pack::types::Integer::from(v))
                        }
                    },
                    "F32Hex" => {
                        use hex::FromHex;
                        let bytes = <[u8;4]>::from_hex(val)?;
                        FogValue::F32(f32::from_be_bytes(bytes))
                    },
                    "F64Hex" => {
                        use hex::FromHex;
                        let bytes = <[u8;8]>::from_hex(val)?;
                        FogValue::F64(f64::from_be_bytes(bytes))
                    },
                    "Bin" => FogValue::Bin(base64_decode(val)?),
                    "Hash" => {
                        let v = fog_pack::types::Hash::from_base58(val).map_err(|_| DecodeError::InvalidBase58)?;
                        FogValue::Hash(v)
                    },
                    "Identity" => {
                        let v = fog_pack::types::Identity::from_base58(val).map_err(|_| DecodeError::InvalidBase58)?;
                        FogValue::Identity(v)
                    },
                    "StreamId" => {
                        let v = fog_pack::types::StreamId::from_base58(val).map_err(|_| DecodeError::InvalidBase58)?;
                        FogValue::StreamId(v)
                    },
                    "LockId" => {
                        let v = fog_pack::types::LockId::from_base58(val).map_err(|_| DecodeError::InvalidBase58)?;
                        FogValue::LockId(v)
                    },
                    "DataLockbox" => {
                        let bytes = base64_decode(val)?;
                        let lockbox = fog_pack::types::DataLockboxRef::from_bytes(&bytes)
                            .map_err(|_| DecodeError::InvalidLockbox)?
                            .to_owned();
                        FogValue::DataLockbox(lockbox)
                    },
                    "IdentityLockbox" => {
                        let bytes = base64_decode(val)?;
                        let lockbox = fog_pack::types::IdentityLockboxRef::from_bytes(&bytes)
                            .map_err(|_| DecodeError::InvalidLockbox)?
                            .to_owned();
                        FogValue::IdentityLockbox(lockbox)
                    },
                    "StreamLockbox" => {
                        let bytes = base64_decode(val)?;
                        let lockbox = fog_pack::types::StreamLockboxRef::from_bytes(&bytes)
                            .map_err(|_| DecodeError::InvalidLockbox)?
                            .to_owned();
                        FogValue::StreamLockbox(lockbox)
                    },
                    "LockLockbox" => {
                        let bytes = base64_decode(val)?;
                        let lockbox = fog_pack::types::LockLockboxRef::from_bytes(&bytes)
                            .map_err(|_| DecodeError::InvalidLockbox)?
                            .to_owned();
                        FogValue::LockLockbox(lockbox)
                    },
                    "Time" => {
                        let time = chrono::DateTime::parse_from_rfc3339(val)?;
                        let sec = time.timestamp();
                        let nano = time.timestamp_subsec_nanos();
                        FogValue::Timestamp(fog_pack::types::Timestamp::from_utc(sec, nano).unwrap())
                    },
                    _ => return Err(DecodeError::UnrecognizedType(ty.to_owned())),
                }
            }
            else {
                FogValue::Str(s.to_owned())
            }
        }
    })
}
