fog-human-json: human-readable fog-pack values
==============================================

This crate provides functions to go back and forth between fog-pack and JSON, 
making it relatively easy for users to view pretty-printed fog-pack values and 
edit them with existing JSON tooling. A common complaint with binary data 
formats like fog-pack is that reading them is painful, and lowering that pain 
with JSON is exactly what this crate is for.

This is *not* a crate for turning regular JSON into fog-pack data. It uses a 
number of special string prefixes to encode fog-pack types in JSON, which can 
interfere with arbitrary JSON-to-fog conversions.

So, what does this actually do for conversion? Well, it takes each fog-pack type 
and either directly converts it to a corresponding JSON type, or it specially 
encodes it in a string that starts with `$fog-`. So a 32-bit floating point 
value could be specifically encoded as `$fog-F32: 1.23`. The full list of types 
is:

- Str: A regular string. This is just prepended so fog-pack strings that start 
	with `$fog-` won't get caught by the parser.
- Bin: Encodes the binary data as Base64 using the "standard" encoding (bonus 
	symbols of `+/`, no padding used, padding is accepted when parsing).
- F32Hex / F64Hex: Encodes a binary32/64 IEEE floating-point value in big-endian hex. 
	The fog-to-json process should only do this when writing out a NaN or 
	Infinity.
- F32 / F64 / Int: Prints a standard JSON Number, but includes the type 
	information. This done by telling the converter to do it specifically, by a 
	user adding type information, or by the converter for any F32 value (as 
	`serde_json` will always use F64 for floating-point).
- Time: Encodes the time as a RFC 3339 formatted string.
- Hash / Identity / StreamId / LockId: Encodes the corresponding primitive as a 
	base58 string (in the Bitcoin base58 style).
- DataLockbox / IdentityLockbox / StreamLockbox / LockLockbox: Encodes the 
	corresponding lockbox as Base64 data, just like with the "Bin" type.

That covers conversion between fog-pack Values and JSON values, but not 
Documents and Entries. Those are converted into JSON objects with the following 
key-value pairs:

- Documents:
	- "schema": If present, a `$fog-Hash:HASH` with the schema.
	- "signer": If present, a `$fog-Identity:IDENTITY` with the signer's 
		Identity. 
	- "compression": If not present, uses default compression. If present and 
		null, no compression is used. If set to a number between 0-255, uses that 
		as the compression level.
	- "data": The document content. Must be present.
- Entries:
	- "parent": Parent document's hash.
	- "key": Entry's string key.
	- "signer": If present, holds the signer's Identity.
	- "compression": If not present, uses default compression. If present and 
		null, no compression is used. If set to a number between 0 & 255, uses that 
		as the compression level.
	- "data": The entry content. Must be present.

When going from JSON to a Document or Entry, if there's a "signer" specified, it 
will attempt to pull a matching IdentityKey from a provided Vault and use that 
to reproduce the signature. If it can't, then the conversion will fail.
