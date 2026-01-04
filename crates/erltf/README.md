# An Erlang Term Format (ETF) Serialization Library for Rust

This crate implements the Erlang Term Format (ETF) serialization format in Rust.

It supports encoding of Rust data structures into the ETF format as well as decoding of
binary ETF-encoded data obtained from an Erlang (or Elixir, or another BEAM-based language) node
into Rust data structures.


## Optional Features

 * `serde`: implements `serde::Serialize` and `serde::Deserialize` for `OwnedTerm`
 * `elixir-interop`: adjusts encoding, decoding behavior to match Elixir conventions (e.g., `Option::None` becomes the `nil` atom instead of `undefined`)


## License

This software is dual-licensed under the MIT License and the Apache License, Version 2.0.

## Copyright

(c) 2025-2026 Michael S. Klishin and Contributors.
