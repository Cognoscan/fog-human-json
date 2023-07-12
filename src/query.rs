use super::*;
use fog_pack::{
    document::NewDocument,
    query::{NewQuery, Query},
    schema::NoSchema,
};

/// Convert a [`NewQuery`] into a JSON value. Can panic if the query is too large.
///
/// The resulting JSON value will be an Object with the following key-value pairs:
///
/// - "validator": The query validator
/// - "key": The query's key, which selects and queries all entries with a matching key.
///
pub fn new_query_to_json(query: &NewQuery) -> JsonValue {
    let doc = NewDocument::new(None, query.validator())
        .expect("Query was way too large");
    let doc = NoSchema::validate_new_doc(doc)
        .expect("Queries should always be valid serializeable fog-pack");
    let validator: FogValueRef = doc.deserialize()
        .expect("All fog-pack documents should be deserializable to a ValueRef");
    
    let mut map: BTreeMap<&str, FogValueRef> = BTreeMap::new();
    map.insert("validator", validator);
    map.insert("key", FogValueRef::Str(query.key()));
    let query = FogValueRef::Map(map);
    fogref_to_json(&query)
}

/// Convert a [`Query`] into a JSON value.
///
/// The resulting JSON value will be an Object with the following key-value pairs:
///
/// - "validator": The query validator
/// - "key": The query's key, which selects and queries all entries with a matching key.
///
pub fn query_to_json(query: &Query) -> JsonValue {
    let doc = NewDocument::new(None, query.validator())
        .expect("Query was way too large, which shouldn't have been possible");
    let doc = NoSchema::validate_new_doc(doc)
        .expect("Queries should always be valid serializeable fog-pack");
    let validator: FogValueRef = doc.deserialize()
        .expect("All fog-pack documents should be deserializable to a ValueRef");
    
    let mut map: BTreeMap<&str, FogValueRef> = BTreeMap::new();
    map.insert("validator", validator);
    map.insert("key", FogValueRef::Str(query.key()));
    let query = FogValueRef::Map(map);
    fogref_to_json(&query)
}

/// Convert JSON into a [`NewQuery`].
///
/// The root JSON value should be an Object with the following key-value pairs:
///
/// - "validator": The query validator
/// - "key": The query's key, which selects and queries all entries with a matching key.
///
pub fn json_to_query(json: &JsonValue) -> Result<NewQuery, ObjectError> {
    let obj = json.as_object().ok_or(ObjectError::NotAnObject)?;

    // Make sure we only have fields we recognize
    for k in obj.keys() {
        match k.as_str() {
            "validator" | "key" => (),
            k => return Err(ObjectError::UnrecognizedKey(k.to_string())),
        }
    }

    // Get the Key
    let key = obj.get("key").ok_or_else(|| ObjectError::MissingKey("key"))?;
    let key = json_to_fog(key)
        .map_err(|e| ObjectError::Decode { key: "key", src: e })?;
    let key = key
        .as_str()
        .ok_or(ObjectError::WrongDataType("key"))?;

    // Get the Validator, which must round-trip through a Document to be encoded
    let validator = obj.get("validator").ok_or_else(|| ObjectError::MissingKey("validator"))?;
    let validator = json_to_fog(validator).map_err(|e| ObjectError::Decode { key: "validator", src: e })?;
    let validator = NewDocument::new(None, validator)?;
    let validator = NoSchema::validate_new_doc(validator)?;
    let validator: fog_pack::validator::Validator = validator.deserialize()?;

    Ok(NewQuery::new(key, validator))
}
