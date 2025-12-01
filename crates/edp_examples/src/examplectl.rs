// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{Context, Result};
use clap::{value_parser, Arg, ArgMatches, Command};
use edp_examples::common;
use edp_node::Node;
use erltf::OwnedTerm;
use tabled::{Table, Tabled};
use tracing_subscriber::EnvFilter;

fn build_cli() -> Command {
    Command::new("examplectl")
        .about("Example Erlang Distribution Protocol CLI")
        .arg(
            Arg::new("node")
                .short('n')
                .long("node")
                .help("RabbitMQ node name (e.g., rabbit@hostname)")
                .required(false),
        )
        .arg(
            Arg::new("longnames")
                .long("longnames")
                .help("Use long node names (fully qualified hostnames)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .help("Operation timeout in seconds")
                .value_parser(value_parser!(u64))
                .default_value("60"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Quiet mode, minimal output")
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(Command::new("listeners").about("List all listeners on the node"))
        .subcommand(
            Command::new("add_vhost")
                .about("Add a new virtual host")
                .arg(
                    Arg::new("vhost")
                        .help("Virtual host name")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("delete_vhost")
                .about("Delete a virtual host")
                .arg(
                    Arg::new("vhost")
                        .help("Virtual host name")
                        .required(true)
                        .index(1),
                ),
        )
        .arg_required_else_help(true)
}

fn unwrap_rpc_response(response: OwnedTerm) -> Result<OwnedTerm> {
    match response {
        OwnedTerm::Tuple(ref tuple) if tuple.len() == 2 => {
            if let OwnedTerm::Atom(ref atom) = tuple[0] {
                if atom.as_ref() == "rex" {
                    return Ok(tuple[1].clone());
                }
            }
            Ok(response)
        }
        _ => Ok(response),
    }
}

#[derive(Tabled)]
struct ListenerRow {
    #[tabled(rename = "Interface")]
    interface: String,
    #[tabled(rename = "Port")]
    port: i64,
    #[tabled(rename = "Protocol")]
    protocol: String,
}

async fn list_listeners(node: &mut Node, target_node: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Listing listeners on {}...", target_node);
    }

    let response = node
        .rpc_call(target_node, "rabbit_networking", "active_listeners", vec![])
        .await
        .context("Failed to call rabbit_networking:active_listeners/0")?;

    let response = unwrap_rpc_response(response)?;

    match response {
        OwnedTerm::List(listeners) => {
            let mut rows = Vec::new();

            for listener in listeners {
                if let OwnedTerm::Tuple(tuple_items) = listener {
                    if tuple_items.len() >= 7 {
                        if let (
                            OwnedTerm::Atom(tag),
                            OwnedTerm::Atom(_node),
                            OwnedTerm::Atom(protocol),
                            _host,
                            OwnedTerm::Tuple(ip_tuple),
                            OwnedTerm::Integer(port),
                            _opts,
                        ) = (
                            &tuple_items[0],
                            &tuple_items[1],
                            &tuple_items[2],
                            &tuple_items[3],
                            &tuple_items[4],
                            &tuple_items[5],
                            &tuple_items[6],
                        ) {
                            if tag.as_ref() != "listener" {
                                continue;
                            }

                            let ip = format_ip_address(ip_tuple);

                            if quiet {
                                println!("{}:{}:{}", protocol, ip, port);
                            } else {
                                rows.push(ListenerRow {
                                    interface: ip,
                                    port: *port,
                                    protocol: protocol.to_string(),
                                });
                            }
                        }
                    }
                }
            }

            if !quiet && !rows.is_empty() {
                let table = Table::new(rows);
                println!("\n{}", table);
            }
        }
        _ => {
            if !quiet {
                println!("Unexpected response format");
            }
        }
    }

    Ok(())
}

fn format_ip_address(ip_tuple: &[OwnedTerm]) -> String {
    let parts: Vec<String> = ip_tuple
        .iter()
        .filter_map(|part| {
            if let OwnedTerm::Integer(n) = part {
                Some(n.to_string())
            } else {
                None
            }
        })
        .collect();

    if parts.len() == 4 {
        parts.join(".")
    } else if parts.len() == 8 {
        parts.join(":")
    } else {
        "::".to_string()
    }
}

async fn add_vhost(node: &mut Node, target_node: &str, vhost: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Adding vhost '{}' on {}...", vhost, target_node);
    }

    let vhost_binary = OwnedTerm::binary(vhost.as_bytes().to_vec());
    let acting_user = OwnedTerm::binary(b"rabbitmqctl".to_vec());

    let response = node
        .rpc_call(
            target_node,
            "rabbit_vhost",
            "add",
            vec![vhost_binary, acting_user],
        )
        .await
        .context("Failed to call rabbit_vhost:add/2")?;

    let response = unwrap_rpc_response(response)?;

    match response {
        OwnedTerm::Atom(atom) if atom.as_ref() == "ok" => {
            if !quiet {
                println!("Successfully added vhost '{}'", vhost);
            }
            Ok(())
        }
        OwnedTerm::Tuple(ref tuple) if tuple.len() == 2 => {
            if let OwnedTerm::Atom(atom) = &tuple[0] {
                if atom.as_ref() == "error" {
                    anyhow::bail!("Failed to add vhost: {:?}", tuple[1]);
                }
            }
            anyhow::bail!("Unexpected response: {:?}", response);
        }
        _ => anyhow::bail!("Unexpected response: {:?}", response),
    }
}

async fn delete_vhost(node: &mut Node, target_node: &str, vhost: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Deleting vhost '{}' from {}...", vhost, target_node);
    }

    let vhost_binary = OwnedTerm::binary(vhost.as_bytes().to_vec());
    let acting_user = OwnedTerm::binary(b"rabbitmqctl".to_vec());

    let response = node
        .rpc_call(
            target_node,
            "rabbit_vhost",
            "delete",
            vec![vhost_binary, acting_user],
        )
        .await
        .context("Failed to call rabbit_vhost:delete/2")?;

    let response = unwrap_rpc_response(response)?;

    match response {
        OwnedTerm::Atom(atom) if atom.as_ref() == "ok" => {
            if !quiet {
                println!("Successfully deleted vhost '{}'", vhost);
            }
            Ok(())
        }
        OwnedTerm::Tuple(ref tuple) if tuple.len() == 2 => {
            if let OwnedTerm::Atom(atom) = &tuple[0] {
                if atom.as_ref() == "error" {
                    anyhow::bail!("Failed to delete vhost: {:?}", tuple[1]);
                }
            }
            anyhow::bail!("Unexpected response: {:?}", response);
        }
        _ => anyhow::bail!("Unexpected response: {:?}", response),
    }
}

async fn run(matches: ArgMatches) -> Result<()> {
    let quiet = matches.get_flag("quiet");
    let longnames = matches.get_flag("longnames");
    let _timeout = matches.get_one::<u64>("timeout").copied().unwrap_or(60);

    if !quiet {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
            )
            .init();
    }

    let cookie = common::get_erlang_cookie().context("Failed to get Erlang cookie")?;
    let hostname = common::get_hostname(longnames)?;

    let target_node = matches
        .get_one::<String>("node")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("rabbit@{}", hostname));

    let remote_host = target_node
        .split_once('@')
        .map(|(_, h)| h)
        .unwrap_or(&hostname);

    let client_node_name = format!("examplectl_{}@{}", std::process::id(), remote_host);

    let mut node = Node::new_hidden(client_node_name, cookie);
    node.start(0).await.context("Failed to start Erlang node")?;

    if !quiet {
        println!("Connecting to {}...", target_node);
    }

    node.connect(&target_node)
        .await
        .context("Failed to connect to RabbitMQ node")?;

    match matches.subcommand() {
        Some(("listeners", _)) => list_listeners(&mut node, &target_node, quiet).await,
        Some(("add_vhost", sub_matches)) => {
            let vhost = sub_matches.get_one::<String>("vhost").unwrap();
            add_vhost(&mut node, &target_node, vhost, quiet).await
        }
        Some(("delete_vhost", sub_matches)) => {
            let vhost = sub_matches.get_one::<String>("vhost").unwrap();
            delete_vhost(&mut node, &target_node, vhost, quiet).await
        }
        _ => {
            anyhow::bail!("Unknown command");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_cli().get_matches();
    run(matches).await
}
