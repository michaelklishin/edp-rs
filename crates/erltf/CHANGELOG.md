# erltf Change Log

## v0.12.0 (in development)

(no changes yet)


## v0.11.0 (Nov 30, 2025)

### Bug Fixes

 * It wasn't possible to "roundtrip" a PID, e.g. during an RPC request-response sequence.

   Due to the `LOCAL_EXT` type tag encoding of PIDs, non-BEAM native implementations must pass
   around additional context for the encoding to be done correctly (the remote node would recognize the right Erlang process).

   This release stores the original `LOCAL_EXT` bytes for PIDs, ports, and references, and reuses
   them when encoding back to the remote node in a response.

### Enhancements

 * `ExternalPid::from_erl_pid_string` for parsing local pid string references (such as `"<0.123.0>"`)
   with additional context that is not available in the input string

 * New `tags` module exposing all ETF tag constants


## v0.10.0 (Nov 29, 2025)

### Enhancements

This release introduces many helper functions for working with common Erlang terms:

 * `OwnedTerm::as_erlang_string_or` for getting charlist/binary/string with a default
 * `OwnedTerm::tuple_get` for safe tuple element access by index
 * `OwnedTerm::tuple_get_string` for getting tuple element as string
 * `OwnedTerm::tuple_get_string_or`
 * `OwnedTerm::tuple_get_atom_string` for getting tuple element atom as string
 * `OwnedTerm::tuple_get_atom_string_or`
 * `OwnedTerm::charlist` for creating charlists (list of integers) from strings
 * `OwnedTerm::is_charlist` predicate (rejects Unicode surrogate codepoints 0xD800-0xDFFF)
 * `OwnedTerm::as_pid` and `OwnedTerm::try_as_pid` for accessing PID terms
 * `OwnedTerm::is_pid` predicate
 * Typed proplist helpers: `proplist_get_i64`, `proplist_get_bool`, `proplist_get_atom`, `proplist_get_string`, `proplist_get_pid`
 * Proplist helpers with defaults: `proplist_get_i64_or`, `proplist_get_bool_or`, `proplist_get_string_or`
 * `OwnedTerm::atom_list` for creating lists of atoms from literals (string slices)
 * `ExternalPid::to_erl_pid_string` for formatting PIDs compatible with `erlang:list_to_pid/1`
 * `ExternalPid::to_charlist_term` for creating charlist terms from PIDs
 * New `Mfa` type for Module/Function/Arity tuples with `Display` and conversion traits
 * Macros that instantiate common Erlang data structures were expanded and are now prefixed with `erl_`:
   - `erl_tuple!` for creating tuple terms
   - `erl_list!` for creating lists
   - `erl_map!` for creating maps
   - `erl_atom!` ditto for single atom terms from string literals
   - `erl_atoms!` for creating a list of atom terms from string literals
   - `erl_int!` for creating an integer term from a numeric literal
 * `OwnedTerm::as_charlist_string` for converting charlists with full Unicode support (0-0x10FFFF)
 * `OwnedTerm::as_list_or_empty` returns list elements or empty slice for non-lists (including `Nil`)
 * `OwnedTerm::try_as_mfa` attempts to parse a term as an MFA triplet
 * `OwnedTerm::format_as_mfa` formats term as "module:function/arity" string
 * `OwnedTerm::format_as_pid` formats PID term as string
 * `OwnedTerm::proplist_get_atom_string` gets proplist atom value as String
 * `OwnedTerm::proplist_get_atom_string_or`
 * `OwnedTerm::proplist_get_pid_string` gets proplist PID value as formatted String
 * `OwnedTerm::proplist_get_mfa_string` gets proplist MFA as "mod:fun/arity" String
 * `OwnedTerm::proplist_get_mfa_string_or`

### Bug Fixes

 * `Mfa::try_from_term` arity is now only accepted as an integer


## v0.9.0 (Nov 29, 2025)

(no changes)


## v0.8.0 (Nov 29, 2025)

### Enhancements

 * New convenience function `into_rex_response` for unwrapping `{rex, Result}` RPC response tuples
 * New helpers for common value comparisons: `is_undefined` and `is_nil_atom` helpers
 * `try_as_*` methods returning `Result`


## v0.6.0 (Nov 29, 2025)

### Enhancements

 * New functions, `OwnedTerm::is_proplist` and `OwnedTerm::is_proplist_element`, for detecting Erlang proplists

 * `OwnedTerm::normalize_proplist` to expand bare atoms to `{Atom, true}` tuples

 * `OwnedTerm::proplist_to_map` and `OwnedTerm::map_to_proplist` for bidirectional conversion
   between proplists and maps

 * `OwnedTerm::to_map_recursive` for recursive conversion of nested proplists to maps,
   similar to `rabbit_data_coercion:to_map_recursive/1` in RabbitMQ

 * `OwnedTerm::atomize_keys` to convert binary/string map or proplist keys to atoms

 * `OwnedTerm::as_list_wrapped` to wrap non-list terms in a list

 * `OwnedTerm::proplist_iter` iterator over proplist key-value pairs


## v0.5.0 (Nov 22, 2025)

### Enhancements

 * Initial public release
 * Erlang External Term Format encoding and decoding
 * Support for all standard Erlang term types
 * Compression support via flate2
