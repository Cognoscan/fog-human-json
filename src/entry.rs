use super::*;

/// Convert an [Entry][fog_pack::entry::Entry] into a JSON Value.
/// 
/// The resulting JSON value will be an Object with at the following key-value pairs:
///
/// - "data": The entry's data
/// - "key": The entry's key, as a string
/// - "parent": The Hash of the parent document
///
/// It may also include a "signer" key, containing the Identity that signed the entry.
pub fn entry_to_json(entry: &fog_pack::entry::Entry) -> JsonValue {
    let data: FogValueRef = entry.deserialize().unwrap();
    let mut map: BTreeMap<&str, FogValueRef> = BTreeMap::new();
    map.insert("data", data);
    map.insert("key", FogValueRef::Str(entry.key()));
    map.insert("parent", FogValueRef::Hash(entry.parent().to_owned()));
    if let Some(signer) = entry.signer() {
        map.insert("signer", FogValueRef::Identity(signer.to_owned()));
    }
    let entry = FogValueRef::Map(map);
    fogref_to_json(&entry)
}

/// Convert a JSON value into a [NewEntry][fog_pack::entry::NewEntry].
///
/// The root JSON value should be an Object with the following key-value pairs:
///
/// - "data": The entry's data
/// - "key": The entry's key, as a string
/// - "parent": The Hash of the parent document
///
/// It may also include:
///
/// - "signer": An Identity to sign the entry with. Conversion fails if the corresponding 
///     IdentityKey cannot be retrieved or used for signing.
/// - "compression": Overrides the default compression settings for the entry. Can be Null or 
///     0-255.
pub fn json_to_entry(json: &JsonValue, vault: Option<&impl fog_crypto::Vault>) -> Result<fog_pack::entry::NewEntry, ObjectError> {
    let obj = json.as_object().ok_or(ObjectError::NotAnObject)?;

    // Make sure we only have fields we recognize
    for k in obj.keys() {
        match k.as_str() {
            "data" | "signer" | "key" | "parent" | "compression" => (),
            k => return Err(ObjectError::UnrecognizedKey(k.to_string())),
        }
    }

    // Fetch & convert the required fields
    let data = obj.get("data").ok_or_else(|| ObjectError::MissingKey("data"))?;
    let data = json_to_fog(data).map_err(|e| ObjectError::Decode { key: "data", src: e })?;
    let key = obj.get("key").ok_or_else(|| ObjectError::MissingKey("key"))?;
    let key = json_to_fog(key)
        .map_err(|e| ObjectError::Decode { key: "key", src: e })?;
    let key = key
        .as_str()
        .ok_or(ObjectError::WrongDataType("key"))?;
    let parent = obj.get("parent").ok_or_else(|| ObjectError::MissingKey("parent"))?;
    let parent = json_to_fog(parent)
        .map_err(|e| ObjectError::Decode { key: "parent", src: e })?;
    let parent = parent
        .as_hash()
        .ok_or(ObjectError::WrongDataType("parent"))?;

    let new_entry = fog_pack::entry::NewEntry::new_ordered(data, key, parent)?;

    // Check the optional compression field
    let new_entry = if let Some(s) = obj.get("compression") {
        match s {
            JsonValue::Null => new_entry.compression(None),
            JsonValue::Number(n) => {
                if let Some(n) = n.as_u64() {
                    let n = u8::try_from(n).map_err(|_| ObjectError::WrongDataType("compression"))?;
                    new_entry.compression(Some(n))
                }
                else {
                    return Err(ObjectError::WrongDataType("compression"));
                }
            },
            _ => return Err(ObjectError::WrongDataType("compression")),
        }
    }
    else { new_entry };

    // Check the optional signer field
    let new_entry = if let Some(s) = obj.get("signer") {
        let s = json_to_fog(s).map_err(|e| ObjectError::Decode { key: "signer", src: e })?
            .as_identity()
            .ok_or(ObjectError::WrongDataType("signer"))?
            .to_owned();
        let Some(vault) = vault else { return Err(ObjectError::NoVault) };
        let Some((_, key)) = vault.find_id(s.clone()) else { return Err(ObjectError::MissingIdentityKey(s.into())) };
        new_entry.sign(&key)?
    }
    else {
        new_entry
    };

    Ok(new_entry)
}
