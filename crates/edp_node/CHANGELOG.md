# edp_node Change Log

## v0.9.0 (in development)

(no changes)


## v0.8.0 (Nov 29, 2025)

### Enhancements

 * `Node::connect_to` and `Node::connect_to_hidden` helpers to reduce connection/node startup verbosity
 * `rpc_call` now auto-unwraps `{rex, Result}` tuples; use `rpc_call_raw` for previous behavior
 * Re-exports `OwnedTerm`, `Atom`, macros, and serde functions from `erltf` and `erltf_serde`


## v0.6.0 (Nov 29, 2025)

### Enhancements

 * Initial public release
 * High-level node abstraction with process management
 * GenServer and GenEvent behavior patterns
 * Process linking and monitoring
