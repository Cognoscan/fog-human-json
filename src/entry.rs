use super::*;
use fog_crypto::identity::IdentityKey;
use fog_pack::{
    types::{Hash, Identity},
    document::Document,
    entry::NewEntry,
};

// Entries require the document they come from...cool....
// The thing to do here, it seems, is to convert the JSON into a parsed thing, then proceed through 
// two additional states

/// Partially converted JSON value that can be completed into a 
/// [NewEntry][fog_pack::entry::NewEntry].
///
/// Conversion is continued by locating the parent document based on the 
/// [`parent`][JsonEntry::parent] hash and providing it to the [`complete`][JsonEntry::complete] 
/// function.
pub struct JsonEntry {
    data: FogValue,
    parent: Hash,
    key: String,
    compression: Option<Option<u8>>,
    signer: Option<Identity>,
}

impl JsonEntry {
    /// Parse a JSON value as part of converting it into an Entry.
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
    ///
    pub fn from_json(json: &JsonValue) -> Result<Self, ObjectError> {
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
            .map_err(|e| ObjectError::Decode { key: "key", src: e })?
            .as_str()
            .ok_or(ObjectError::WrongDataType("key"))?
            .to_owned();
        let parent = obj.get("parent").ok_or_else(|| ObjectError::MissingKey("parent"))?;
        let parent = json_to_fog(parent)
            .map_err(|e| ObjectError::Decode { key: "parent", src: e })?
            .as_hash()
            .ok_or(ObjectError::WrongDataType("parent"))?
            .to_owned();

        // Check the optional compression field
        let compression = if let Some(s) = obj.get("compression") {
            match s {
                JsonValue::Null => Some(None),
                JsonValue::Number(n) => {
                    if let Some(n) = n.as_u64() {
                        let n = u8::try_from(n).map_err(|_| ObjectError::WrongDataType("compression"))?;
                        Some(Some(n))
                    }
                    else {
                        return Err(ObjectError::WrongDataType("compression"));
                    }
                },
                _ => return Err(ObjectError::WrongDataType("compression")),
            }
        }
        else { None };

        // Check the optional signer field
        let signer = if let Some(s) = obj.get("signer") {
            let s = json_to_fog(s).map_err(|e| ObjectError::Decode { key: "signer", src: e })?
                .as_identity()
                .ok_or(ObjectError::WrongDataType("signer"))?
                .to_owned();
            Some(s)
        }
        else { None };

        Ok(Self {
            data,
            key,
            parent,
            compression,
            signer,
        })
    }

    /// Get the hash of the parent document.
    pub fn parent(&self) -> &Hash {
        &self.parent
    }

    /// Attempt to complete the [`NewEntry`] by providing the parent [`Document`].
    pub fn complete(self, parent: &Document) -> Result<MaybeEntry, ObjectError> {
        let entry = fog_pack::entry::NewEntry::new_ordered(self.data, self.key.as_str(), parent)?;
        let entry = if let Some(compression) = self.compression {
            entry.compression(compression)
        }
        else {
            entry
        };
        let ok = if let Some(signer) = self.signer {
            MaybeEntry::SignEntry(SignEntry { entry, signer })
        }
        else {
            MaybeEntry::NewEntry(entry)
        };
        Ok(ok)
    }
}

/// A [`NewEntry`] that may still require signing.
pub enum MaybeEntry {
    /// A completed [`NewEntry`]
    NewEntry(NewEntry),
    /// A [`NewEntry`] that must first be signed
    SignEntry(SignEntry),
}

/// An almost completed [`NewEntry`]. Complete it by finding the appropriate 
/// [`IdentityKey`][IdentityKey] and calling 
/// [`complete`][SignEntry::complete].
pub struct SignEntry {
    entry: NewEntry,
    signer: Identity,
}

impl SignEntry {

    /// Get the Identity that should sign this.
    pub fn signer(&self) -> &Identity {
        &self.signer
    }

    /// Attempt to sign the Entry and complete it.
    pub fn complete(self, key: &IdentityKey) -> Result<NewEntry, ObjectError> {
        if key.id() != &self.signer {
            Err(ObjectError::IncorrectIdentityKey(Box::new(self.signer)))
        }
        else {
            Ok(self.entry.sign(key)?)
        }
    }
}


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

