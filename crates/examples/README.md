# Erlang Distribution Protocol Client in Rust: the Examples

This crate contains some executable examples for this suite of libraries.

## Prerequisites

 * A modern Rust toolchain
 * For examples that use RabbitMQ nodes: a running RabbitMQ node of a recent version (e.g. a `4.2.x`)
 * An Erlang cookie file at `~/.erlang.cookie`

## Examples

### 1. EPMD Names (`example_epmd_names`)

Queries the local EPMD daemon for the list of registered Erlang nodes, that is, what `epmd -names` reports.

**Usage:**

Bash/Zsh:
```bash
cargo run --package examples --bin example_epmd_names
```

Nu shell:
```nu
cargo run --package examples --bin example_epmd_names
```

**Expected Output:**

```
name rabbit at port 25672
name some_other_node at port 45678
```

---

### 2. RabbitMQ Status (`example_rabbitmq_status`)

Connects to a RabbitMQ node and calls `rabbit:status/0` via the Erlang Distribution Protocol, demonstrating full end-to-end RPC functionality.

**Usage:**

Bash/Zsh:
```bash
# Connect to default node (rabbit@<hostname>)
cargo run --package examples --bin example_rabbitmq_status

# Connect to a specific node
cargo run --package examples --bin example_rabbitmq_status rabbit@my-server
```

Nu shell:
```nu
# Connect to default node (rabbit@<hostname>)
cargo run --package examples --bin example_rabbitmq_status

# Connect to a specific node
cargo run --package examples --bin example_rabbitmq_status rabbit@my-server
```

**Arguments:**

- `[NODE_NAME]` (optional): The Erlang node name to connect to. Defaults to `rabbit@<hostname>` where `<hostname>` is automatically detected.

**Expected Output:**

```
Connecting to RabbitMQ node: rabbit@localhost
Using cookie from ~/.erlang.cookie
Connected successfully!
Calling rabbit:status/0...
RPC call sent, waiting for response...

Response from rabbit:status/0:
Tuple([
    Atom(Atom { name: "ok" }),
    List([
        Tuple([
            Atom(Atom { name: "pid" }),
            Integer(12345),
        ]),
        Tuple([
            Atom(Atom { name: "running_applications" }),
            List([...]),
        ]),
        ...
    ])
])
```

**Troubleshooting:**

1. **Connection refused**: Ensure RabbitMQ is running and listening on the distribution port (default: 25672)
2. **Authentication failed**: Verify your `~/.erlang.cookie` matches the RabbitMQ node's cookie
3. **EPMD lookup failed**: Check that EPMD is running (`epmd -names` should work)
4. **Invalid node name**: Ensure the node name format is `name@host`

## Building Examples

Bash/Zsh:
```bash
cargo build --package examples
```

Nu shell:
```nu
cargo build --package examples
```

## Running Tests

The examples themselves serve as integration tests for the EDP client library.

---

### 3. Simple Node (`example_simple_node`)

Demonstrates creating a distributed Erlang node with processes, message passing, and OTP patterns.

**Usage:**

```bash
cargo run --package examples --bin example_simple_node
```

## Development

To add new examples:

1. Create a new source file in `src/`
2. Add a `[[bin]]` section to `Cargo.toml`
3. Update this README with usage instructions

## License

Copyright (C) 2025-2026 Michael S. Klishin and Contributors

Licensed under the Apache License, Version 2.0
