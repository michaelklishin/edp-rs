# erltf Change Log

## v0.7.0 (in development)

No changes yet.


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
