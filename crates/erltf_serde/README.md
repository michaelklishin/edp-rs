# A Serde Library for the Erlang Term Format

This crate provides Serde integration for `erltf`, making it easy to serialize and deserialize
Rust structs to and from Erlang Term Format.


## Optional Features

 * `elixir-interop`: adjusts encoding, decoding behavior to match Elixir conventions (e.g., `Option::None` becomes the `nil` atom instead of `undefined`)


## License

This software is dual-licensed under the MIT License and the Apache License, Version 2.0.

## Copyright

(c) 2025-2026 Michael S. Klishin and Contributors.
