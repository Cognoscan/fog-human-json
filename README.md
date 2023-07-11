fog-human-json: human-readable fog-pack values
==============================================

This library provides functions to go back and forth between fog-pack and JSON, 
making it relatively easy for users to view pretty-printed fog-pack values and 
edit them with existing JSON tooling. A common complaint with binary data 
formats like fog-pack is that reading them is painful, and lowering that pain 
with JSON is exactly what this library is for.

This is *not* a library for turning regular JSON into fog-pack data. It uses a 
number of special string prefixes to encode fog-pack types in JSON, which can 
interfere with arbitrary JSON-to-fog conversions.
