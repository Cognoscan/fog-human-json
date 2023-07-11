use super::*;

fn base64_encode<T: AsRef<[u8]>>(input: T, output_buf: &mut String) {
    use base64::engine::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode_string(input, output_buf)
}

/// Convert a fog-pack value to a JSON Value.
pub fn fog_to_json (val: &FogValue) -> JsonValue {
    match val {
        FogValue::Null => JsonValue::Null,
        FogValue::Bool(b) => JsonValue::Bool(*b),
        FogValue::Int(i) => if let Some(i) = i.as_u64() {
            JsonValue::Number(JsonNumber::from(i))
        } else {
            JsonValue::Number(JsonNumber::from(i.as_i64().unwrap()))
        },
        FogValue::Str(s) => if s.starts_with(FOG_PREFIX) {
            const STR_PREFIX: &str = "$fog-Str:";
            let mut new_s = String::with_capacity(s.len() + STR_PREFIX.len());
            new_s.push_str(STR_PREFIX);
            new_s.push_str(s);
            JsonValue::String(new_s)
        } else {
            JsonValue::String(s.clone())
        },
        FogValue::F32(f) => {
            const F32_PREFIX: &str = "$fog-F32:";
            const F32HEX_PREFIX: &str = "$fog-F32Hex:";
            if f.is_finite() {
                let mut s = String::from(F32_PREFIX);
                let mut buf = ryu::Buffer::new();
                s.push_str(buf.format_finite(*f));
                JsonValue::String(s)
            }
            else {
                let mut s = String::from(F32HEX_PREFIX);
                let v = hex::encode(f.to_be_bytes());
                s.push_str(&v);
                JsonValue::String(s)
            }
        },
        FogValue::F64(f) => {
            const F64HEX_PREFIX: &str = "$fog-F64Hex:";
            if let Some(n) = JsonNumber::from_f64(*f) {
                JsonValue::Number(n)
            }
            else {
                let mut s = String::from(F64HEX_PREFIX);
                let v = hex::encode(f.to_be_bytes());
                s.push_str(&v);
                JsonValue::String(s)
            }
        },
        FogValue::Bin(b) => {
            let mut s = String::from("$fog-Bin:");
            base64_encode(b, &mut s);
            JsonValue::String(s)
        },
        FogValue::Map(map) => {
            let mut obj = JsonMap::new();
            for (k, v) in map.iter() {
                obj.insert(k.clone(), fog_to_json(v));
            }
            JsonValue::Object(obj)
        }
        FogValue::Array(array) => {
            let array: Vec<JsonValue> = array.iter().map(fog_to_json).collect();
            JsonValue::Array(array)
        },
        FogValue::Hash(v) => {
            let mut s = String::from("$fog-Hash:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValue::Identity(v) => {
            let mut s = String::from("$fog-Identity:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValue::StreamId(v) => {
            let mut s = String::from("$fog-StreamId:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValue::LockId(v) => {
            let mut s = String::from("$fog-LockId:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValue::DataLockbox(v) => {
            let mut s = String::from("$fog-DataLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValue::IdentityLockbox(v) => {
            let mut s = String::from("$fog-IdentityLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValue::StreamLockbox(v) => {
            let mut s = String::from("$fog-StreamLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValue::LockLockbox(v) => {
            let mut s = String::from("$fog-LockLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValue::Timestamp(t) => {
            use chrono::offset::TimeZone;
            let mut s = String::from("$fog-Time:");
            let time = chrono::Utc.timestamp_opt(
                t.timestamp_utc(), t.timestamp_subsec_nanos()
            ).unwrap();
            let t = time.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true);
            s.push_str(&t);
            JsonValue::String(s)
        }
    }
}

/// Convert a fog-pack ValueRef to a JSON Value.
pub fn fogref_to_json (val: &FogValueRef) -> JsonValue {
    match val {
        FogValueRef::Null => JsonValue::Null,
        FogValueRef::Bool(b) => JsonValue::Bool(*b),
        FogValueRef::Int(i) => if let Some(i) = i.as_u64() {
            JsonValue::Number(JsonNumber::from(i))
        } else {
            JsonValue::Number(JsonNumber::from(i.as_i64().unwrap()))
        },
        FogValueRef::Str(s) => if s.starts_with(FOG_PREFIX) {
            const STR_PREFIX: &str = "$fog-Str:";
            let mut new_s = String::with_capacity(s.len() + STR_PREFIX.len());
            new_s.push_str(STR_PREFIX);
            new_s.push_str(s);
            JsonValue::String(new_s)
        } else {
            JsonValue::String(s.to_string())
        },
        FogValueRef::F32(f) => {
            const F32_PREFIX: &str = "$fog-F32:";
            const F32HEX_PREFIX: &str = "$fog-F32Hex:";
            if f.is_finite() {
                let mut s = String::from(F32_PREFIX);
                let mut buf = ryu::Buffer::new();
                s.push_str(buf.format_finite(*f));
                JsonValue::String(s)
            }
            else {
                let mut s = String::from(F32HEX_PREFIX);
                let v = hex::encode(f.to_be_bytes());
                s.push_str(&v);
                JsonValue::String(s)
            }
        },
        FogValueRef::F64(f) => {
            const F64HEX_PREFIX: &str = "$fog-F64Hex:";
            if let Some(n) = JsonNumber::from_f64(*f) {
                JsonValue::Number(n)
            }
            else {
                let mut s = String::from(F64HEX_PREFIX);
                let v = hex::encode(f.to_be_bytes());
                s.push_str(&v);
                JsonValue::String(s)
            }
        },
        FogValueRef::Bin(b) => {
            let mut s = String::from("$fog-Bin:");
            base64_encode(b, &mut s);
            JsonValue::String(s)
        },
        FogValueRef::Map(map) => {
            let mut obj = JsonMap::new();
            for (k, v) in map.iter() {
                obj.insert(k.to_string(), fogref_to_json(v));
            }
            JsonValue::Object(obj)
        }
        FogValueRef::Array(array) => {
            let array: Vec<JsonValue> = array.iter().map(fogref_to_json).collect();
            JsonValue::Array(array)
        },
        FogValueRef::Hash(v) => {
            let mut s = String::from("$fog-Hash:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValueRef::Identity(v) => {
            let mut s = String::from("$fog-Identity:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValueRef::StreamId(v) => {
            let mut s = String::from("$fog-StreamId:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValueRef::LockId(v) => {
            let mut s = String::from("$fog-LockId:");
            let v = v.to_base58();
            s.push_str(&v);
            JsonValue::String(s)
        },
        FogValueRef::DataLockbox(v) => {
            let mut s = String::from("$fog-DataLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValueRef::IdentityLockbox(v) => {
            let mut s = String::from("$fog-IdentityLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValueRef::StreamLockbox(v) => {
            let mut s = String::from("$fog-StreamLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValueRef::LockLockbox(v) => {
            let mut s = String::from("$fog-LockLockbox:");
            base64_encode(v.as_bytes(), &mut s);
            JsonValue::String(s)
        },
        FogValueRef::Timestamp(t) => {
            use chrono::offset::TimeZone;
            let mut s = String::from("$fog-Time:");
            let time = chrono::Utc.timestamp_opt(
                t.timestamp_utc(), t.timestamp_subsec_nanos()
            ).unwrap();
            let t = time.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true);
            s.push_str(&t);
            JsonValue::String(s)
        }
    }
}
