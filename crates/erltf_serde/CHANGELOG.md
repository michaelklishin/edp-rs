# erltf_serde Change Log

## v0.9.0 (in development)

(no changes)


## v0.8.0 (Nov 29, 2025)

### Enhancements

 * New trait, `OwnedTermExt`, provides two functions: `try_deserialize` and `try_deserialize_proplist`


## v0.6.0 (Nov 29, 2025)

### Enhancements

 * Enhanced deserialization support for proplists: they are converted to maps using `OwnedTerm::to_map_recursive`


## v0.5.0 (Nov 22, 2025)

### Enhancements

 * Initial public release
 * Serde integration for Erlang External Term Format
 * Serialization and deserialization between Rust types and Erlang terms
