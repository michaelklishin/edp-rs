# edp_node Change Log

## v0.11.0 (in development)

(no changes yet)


## v0.10.0 (Nov 29, 2025)

### Enhancements

 * New `erlang_mod_fns` module with RPC helpers: `erlang_system_info`, `erlang_statistics`, `erlang_memory`, `erlang_processes`, `erlang_process_info`, `erlang_list_to_pid`
 * Re-exports `ExternalPid` and `Mfa` from `erltf`
 * Re-exports all `erl_*` macros from `erltf`: `erl_tuple!`, `erl_list!`, `erl_map!`, `erl_atom!`, `erl_atoms!`, `erl_int!`


## v0.9.0 (Nov 29, 2025)

### Enhancements

 * New `rpc_call_with_timeout` and `rpc_call_raw_with_timeout` functions for RPC calls with custom timeout
 * New default timeout constant, `DEFAULT_RPC_TIMEOUT`, defaults to 10 seconds
 * `RpcTimeout` error now includes the timeout duration in its message


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
