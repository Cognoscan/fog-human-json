use fog_crypto::identity::{Identity, IdentityKey};
use fog_pack::document::{Document, NewDocument};

use super::*;

/// Convert a [Document] into a JSON Value.
///
/// The resulting JSON value will be an Object with at least a "data" key present, containing the 
/// data from the Document. Optional key-value pairs are:
///
/// - "schema": A fog-pack Hash of the schema used by the document.
/// - "signer": A fog-pack Identity that signed the document.
pub fn doc_to_json(doc: &Document) -> JsonValue {
    // Deserializing to a fog-pack ValueRef should never fail
    let data: FogValueRef = doc.deserialize().unwrap();
    let mut map: BTreeMap<&str, FogValueRef> = BTreeMap::new();
    map.insert("data", data);
    if let Some(signer) = doc.signer() {
        map.insert("signer", FogValueRef::Identity(signer.to_owned()));
    }
    if let Some(schema) = doc.schema_hash() {
        map.insert("schema", FogValueRef::Hash(schema.to_owned()));
    }
    let doc = FogValueRef::Map(map);
    fogref_to_json(&doc)
}

/// A [`NewDocument`] that may still require signing.
pub enum MaybeDocument {
    /// A completed [`NewDocument`]
    NewDocument(NewDocument),
    /// A [`NewDocument`] that must first be signed
    SignDocument(SignDocument),
}

/// An almost completed [`NewDocument`]. Complete it by finding the appropriate 
/// [`IdentityKey`][IdentityKey] and calling [`complete`][SignDocument::complete].
pub struct SignDocument {
    doc: NewDocument,
    signer: Identity,
}

impl SignDocument {

    /// Get the Identity that should sign this.
    pub fn signer(&self) -> &Identity {
        &self.signer
    }

    /// Attempt to sign the Document and complete it.
    pub fn complete(self, key: &IdentityKey) -> Result<NewDocument, ObjectError> {
        if key.id() != &self.signer {
            Err(ObjectError::IncorrectIdentityKey(Box::new(self.signer)))
        }
        else {
            Ok(self.doc.sign(key)?)
        }
    }
}

/// Convert a JSON value into a [`NewDocument`].
///
/// The root JSON value should be an Object with at least a "data" key present. Optional key-value 
/// pairs are:
///
/// - "schema": A fog-pack Hash of the schema to use for the document.
/// - "signer": A fog-pack Identity to use for signing the document.
/// - "compression": Overrides the default compression settings for the document. Can be Null or 
///     0-255.
///
/// If signing is required, this returns a [`SignDocument`] in an enum, which must first be signed 
/// before completion.
pub fn json_to_doc(json: &JsonValue) -> Result<MaybeDocument, ObjectError> {
    let obj = json.as_object().ok_or(ObjectError::NotAnObject)?;

    // Make sure we only have fields we recognize
    for k in obj.keys() {
        match k.as_str() {
            "data" | "signer" | "schema" | "compression" => (),
            k => return Err(ObjectError::UnrecognizedKey(k.to_string())),
        }
    }

    // Fetch & convert fields for making the document
    let data = obj.get("data").ok_or_else(|| ObjectError::MissingKey("data"))?;
    let data = json_to_fog(data).map_err(|e| ObjectError::Decode { key: "data", src: e })?;
    let schema = if let Some(s) = obj.get("schema") {
        let s = json_to_fog(s).map_err(|e| ObjectError::Decode { key: "schema", src: e })?
            .as_hash()
            .ok_or(ObjectError::WrongDataType("schema"))?
            .to_owned();
        Some(s)
    }
    else { 
        None
    };
    let new_doc = fog_pack::document::NewDocument::new_ordered(data, schema.as_ref())?;

    // Check the optional compression field
    let new_doc = if let Some(s) = obj.get("compression") {
        match s {
            JsonValue::Null => new_doc.compression(None),
            JsonValue::Number(n) => {
                if let Some(n) = n.as_u64() {
                    let n = u8::try_from(n).map_err(|_| ObjectError::WrongDataType("compression"))?;
                    new_doc.compression(Some(n))
                }
                else {
                    return Err(ObjectError::WrongDataType("compression"));
                }
            },
            _ => return Err(ObjectError::WrongDataType("compression")),
        }
    }
    else { new_doc };

    // Check the optional signer field
    if let Some(s) = obj.get("signer") {
        let s = json_to_fog(s).map_err(|e| ObjectError::Decode { key: "signer", src: e })?
            .as_identity()
            .ok_or(ObjectError::WrongDataType("signer"))?
            .to_owned();
        Ok(MaybeDocument::SignDocument(SignDocument { doc: new_doc, signer: s }))
    }
    else {
        Ok(MaybeDocument::NewDocument(new_doc))
    }
}
