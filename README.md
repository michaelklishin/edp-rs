# Erlang Distribution Protocol Client Toolkit for Rust

This is a set of Rust libraries that implement the [Erlang External Term Format](https://www.erlang.org/docs/27/apps/erts/erl_ext_dist.html) and the [Erlang Distribution Protocol](https://www.erlang.org/docs/27/apps/erts/erl_dist_protocol).

### Inspiration

This project was heavily inspired by [a set of Go libraries](https://github.com/goerlang).


### Key Differences from Similar Libraries

 * Extensive test coverage, including unit, integration, and property-based tests
 * `crates/erltf` implements supports for [fragmented messages](https://www.erlang.org/docs/27/apps/erts/erl_ext_dist#distribution-header) (a.k.a. [`DFLAG_FRAGMENTS`](https://www.erlang.org/docs/27/apps/erts/erl_dist_protocol#DFLAG_FRAGMENTS)), a feature
   almost always skipped by other implementations due to its complexity and imperfect documentation
 * `crates/erltf_serde` provides Serde glue for `erltf`
 * `crates/erltf_serde_derive`: `derive`-oriented Serde glue for `erltf`
 * `crates/edp_client` and `crates/edp_node` provide higher-level abstractions


## Project Maturity

This set of libraries is very young. Breaking API changes are fairly likely.


## Target Erlang/OTP Versions

This set of libraries target Erlang/OTP 26 and 27. It should be compatible with Erlang 28.


## Subprojects

 * `crates/edp_client`: an Erlang Distribution Protocol client using Tokio
 * `crates/erltf`: an Erlang Term Format implementation
 * `crates/erltf_serde`: Serde glue for `erltf`
 * `crates/edp_examples`: various examples that demonstrate the usage of this library suite


## Examples

 * [`ets-browser`](https://github.com/michaelklishin/ets-browser) implements an ETS table browser tool using these libraries
 * A number of integration examples can be found under `crates/edp_examples`.


## Throughput and Efficiency

These libraries were developed with efficiency in mind, both in terms of memory allocations and binary
parser performance.

The Erlang Term Format encoder and decoder can achive throughput ranging from hundreds of MiBs to tens of GiBs
per second, see `CONTRIBUTING.md` to learn how to run the benchmarks.

Actual throughput will vary from workload to workload.



## Contributing

Contributions to this project are very welcome. Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) to learn more.


## License

This software is dual-licensed under the MIT License and the Apache License, Version 2.0.


## Copyright

(c) 2025-2026 Michael S. Klishin and Contributors.
