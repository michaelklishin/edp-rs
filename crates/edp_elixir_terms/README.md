# Elixir Term Construction Helpers

Helpers for constructing Elixir-compatible terms. These are building blocks
for EDP interop, not a full Elixir implementation.

## Features

 * Builders for keyword lists and atom-keyed maps
 * Elixir struct representations: `Range`, `MapSet`, `Date`, `Time`, `DateTime`, `NaiveDateTime`
 * GenServer message tuple helpers
 * Exception struct types

## Example

```rust
use edp_elixir_terms::{KeywordListBuilder, ElixirRange, ElixirMapSet};

// Build a keyword list
let opts = KeywordListBuilder::new()
    .put("timeout", 5000)
    .put("retry", true)
    .build();

// Create a range
let range = ElixirRange::ascending(1, 10);
for i in range {
    println!("{}", i);
}

// Work with MapSets
let mut set = ElixirMapSet::new();
set.insert(erltf::OwnedTerm::integer(1));
set.insert(erltf::OwnedTerm::integer(2));
```

## License

This software is dual-licensed under the MIT License and the Apache License, Version 2.0.

## Copyright

(c) 2025-2026 Michael S. Klishin and Contributors.
