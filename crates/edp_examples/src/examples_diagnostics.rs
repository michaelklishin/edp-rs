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
use std::collections::HashMap;
use tabled::{Table, Tabled};
use tracing_subscriber::EnvFilter;

fn build_cli() -> Command {
    Command::new("examples-diagnostics")
        .about("Example Erlang Distribution Protocol diagnostics CLI")
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
        .subcommand(
            Command::new("list_queues")
                .about("List queues and their properties")
                .arg(
                    Arg::new("vhost")
                        .short('p')
                        .long("vhost")
                        .help("Virtual host")
                        .required(false),
                )
                .arg(
                    Arg::new("columns")
                        .help("Queue info items to display (name, messages, consumers, memory)")
                        .num_args(0..)
                        .required(false),
                ),
        )
        .subcommand(Command::new("log_location").about("Show log file location"))
        .subcommand(Command::new("status").about("Display node status (calls rabbit:status/0)"))
        .subcommand(
            Command::new("product_info")
                .about("Display product information (calls rabbit:product_info/0)"),
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

fn parse_proplist(props: &[OwnedTerm]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for term in props {
        if let OwnedTerm::Tuple(tuple) = term {
            if tuple.len() == 2 {
                if let OwnedTerm::Atom(key) = &tuple[0] {
                    map.insert(key.to_string(), term_to_string(&tuple[1]));
                }
            }
        }
    }
    map
}

fn term_to_string(term: &OwnedTerm) -> String {
    match term {
        OwnedTerm::Atom(s) => s.to_string(),
        OwnedTerm::Binary(b) => String::from_utf8_lossy(b).to_string(),
        OwnedTerm::Integer(n) => n.to_string(),
        OwnedTerm::Float(f) => f.to_string(),
        OwnedTerm::List(items) => {
            if let Some(string) = try_list_as_string(items) {
                string
            } else {
                let parts: Vec<String> = items.iter().map(term_to_string).collect();
                format!("[{}]", parts.join(", "))
            }
        }
        OwnedTerm::Tuple(items) if items.len() == 4 => {
            if let (OwnedTerm::Atom(tag), OwnedTerm::Binary(_vhost), OwnedTerm::Atom(kind), name) =
                (&items[0], &items[1], &items[2], &items[3])
            {
                if tag.as_ref() == "resource" && kind.as_ref() == "queue" {
                    return term_to_string(name);
                }
            }
            format!("{:?}", term)
        }
        OwnedTerm::Tuple(items) if items.len() == 2 => {
            if let (OwnedTerm::Atom(_), value) = (&items[0], &items[1]) {
                term_to_string(value)
            } else {
                format!("{:?}", term)
            }
        }
        _ => format!("{:?}", term),
    }
}

fn try_list_as_string(items: &[OwnedTerm]) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    let mut bytes = Vec::new();
    for item in items {
        if let OwnedTerm::Integer(n) = item {
            if *n >= 0 && *n <= 255 {
                bytes.push(*n as u8);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    String::from_utf8(bytes).ok()
}

#[derive(Tabled)]
struct QueueRow {
    name: String,
    messages: String,
    consumers: String,
}

async fn list_queues(
    node: &mut Node,
    target_node: &str,
    vhost: Option<String>,
    columns: Vec<String>,
    quiet: bool,
) -> Result<()> {
    let vhost_name = vhost.unwrap_or_else(|| "/".to_string());

    if !quiet {
        println!(
            "Listing queues in vhost '{}' on {}...",
            vhost_name, target_node
        );
    }

    let vhost_binary = OwnedTerm::binary(vhost_name.as_bytes().to_vec());

    let use_default_columns = columns.is_empty();
    let queue_info_items = if use_default_columns {
        vec![
            OwnedTerm::atom("name"),
            OwnedTerm::atom("messages"),
            OwnedTerm::atom("consumers"),
        ]
    } else {
        columns.iter().map(OwnedTerm::atom).collect()
    };

    let response = node
        .rpc_call(
            target_node,
            "rabbit_amqqueue",
            "info_all",
            vec![vhost_binary, OwnedTerm::List(queue_info_items.clone())],
        )
        .await
        .context("Failed to call rabbit_amqqueue:info_all/2")?;

    let response = unwrap_rpc_response(response)?;

    match response {
        OwnedTerm::Nil => {
            if !quiet {
                println!("No queues found");
            }
            return Ok(());
        }
        OwnedTerm::List(queues) => {
            if queues.is_empty() {
                if !quiet {
                    println!("No queues found");
                }
                return Ok(());
            }

            if use_default_columns && !quiet {
                let mut rows = Vec::new();
                for queue in queues {
                    if let OwnedTerm::List(props) = queue {
                        let props_map = parse_proplist(&props);
                        rows.push(QueueRow {
                            name: props_map.get("name").cloned().unwrap_or_default(),
                            messages: props_map.get("messages").cloned().unwrap_or_default(),
                            consumers: props_map.get("consumers").cloned().unwrap_or_default(),
                        });
                    }
                }
                let table = Table::new(rows);
                println!("\n{}", table);
            } else {
                let header: Vec<String> = if columns.is_empty() {
                    ["name", "messages", "consumers"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    columns.clone()
                };

                if !quiet {
                    println!("\n{}", header.join("\t"));
                }

                for queue in queues {
                    if let OwnedTerm::List(props) = queue {
                        let props_map = parse_proplist(&props);
                        let values: Vec<String> = header
                            .iter()
                            .map(|col| props_map.get(col).cloned().unwrap_or_default())
                            .collect();
                        println!("{}", values.join("\t"));
                    }
                }
            }
        }
        _ => {
            if !quiet {
                println!("Unexpected response format: {:?}", response);
            }
        }
    }

    Ok(())
}

async fn log_location(node: &mut Node, target_node: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Getting log file location from {}...", target_node);
    }

    let response = node
        .rpc_call(target_node, "rabbit", "log_locations", vec![])
        .await
        .context("Failed to call rabbit:log_locations/0")?;

    let response = unwrap_rpc_response(response)?;

    match response {
        OwnedTerm::Binary(path) => {
            let path_str = String::from_utf8_lossy(&path);
            if quiet {
                println!("{}", path_str);
            } else {
                println!("\nLog file location: {}", path_str);
            }
        }
        OwnedTerm::Atom(path) => {
            if quiet {
                println!("{}", path);
            } else {
                println!("\nLog file location: {}", path);
            }
        }
        OwnedTerm::List(paths) if !paths.is_empty() => {
            if !quiet {
                println!("\nLog files:");
            }
            for path in paths {
                println!("  {}", term_to_string(&path));
            }
        }
        _ => {
            if quiet {
                println!("unknown");
            } else {
                println!("\nLog file location: {:?}", response);
            }
        }
    }

    Ok(())
}

async fn status(node: &mut Node, target_node: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Getting node status from {}...", target_node);
    }

    let response = node
        .rpc_call(target_node, "rabbit", "status", vec![])
        .await
        .context("Failed to call rabbit:status/0")?;

    let response = unwrap_rpc_response(response)?;

    if quiet {
        println!("{:?}", response);
        return Ok(());
    }

    println!("\nStatus of node {}", target_node);
    println!("{}", "=".repeat(40));

    if let OwnedTerm::List(_) = &response {
        if let Some(running_apps) = response.proplist_get_atom_key("running_applications") {
            if let Some(apps) = running_apps.as_list() {
                for app in apps {
                    if let Some(app_tuple) = app.as_tuple() {
                        if app_tuple.len() == 3 {
                            let app_name = app_tuple[0].atom_name().unwrap_or("unknown");
                            if app_name == "rabbit" {
                                let version = app_tuple[2]
                                    .as_erlang_string()
                                    .unwrap_or_else(|| "unknown".to_string());
                                println!("\nRuntime\n");
                                println!("RabbitMQ version: {}", version);
                            }
                        }
                    }
                }
            }
        }

        if let Some(config_files) = response.proplist_get_atom_key("config_files") {
            println!("\nConfig files\n");
            if let Some(files) = config_files.as_list() {
                if files.is_empty() {
                    println!(" * (none)");
                } else {
                    for file in files {
                        if let Some(path) = file.as_erlang_string() {
                            println!(" * {}", path);
                        }
                    }
                }
            }
        }

        if let Some(log_files) = response.proplist_get_atom_key("log_files") {
            println!("\nLog file(s)\n");
            if let Some(files) = log_files.as_list() {
                if files.is_empty() {
                    println!(" * (none)");
                } else {
                    for file in files {
                        if let Some(file_tuple) = file.as_tuple() {
                            if file_tuple.len() == 2 {
                                let log_type = file_tuple[0].atom_name().unwrap_or("unknown");
                                if let Some(path) = file_tuple[1].as_erlang_string() {
                                    println!(" * {} {}", log_type, path);
                                }
                            }
                        } else if let Some(path) = file.as_erlang_string() {
                            println!(" * {}", path);
                        }
                    }
                }
            }
        }

        if let Some(cluster_name) = response.proplist_get_atom_key("cluster_name") {
            if let Some(name) = cluster_name.as_erlang_string() {
                println!("\nCluster name: {}", name);
            }
        }

        if let Some(data_dir) = response.proplist_get_atom_key("data_directory") {
            if let Some(dir) = data_dir.as_erlang_string() {
                println!("Data directory: {}", dir);
            }
        }

        if let Some(running_apps) = response.proplist_get_atom_key("running_applications") {
            println!("\nEnabled plugins\n");
            if let Some(apps) = running_apps.as_list() {
                let mut plugins = Vec::new();
                for app in apps {
                    if let Some(app_tuple) = app.as_tuple() {
                        if app_tuple.len() == 3 {
                            let app_name = app_tuple[0].atom_name().unwrap_or("unknown");
                            if app_name.starts_with("rabbitmq_") {
                                plugins.push(app_name.to_string());
                            }
                        }
                    }
                }
                if plugins.is_empty() {
                    println!(" * (none)");
                } else {
                    for plugin in plugins {
                        println!(" * {}", plugin);
                    }
                }
            }
        }

        if let Some(alarms) = response.proplist_get_atom_key("alarms") {
            println!("\nAlarms\n");
            if let Some(alarm_list) = alarms.as_list() {
                if alarm_list.is_empty() {
                    println!(" * (none)");
                } else {
                    for alarm in alarm_list {
                        println!(" * {:?}", alarm);
                    }
                }
            }
        }

        if let Some(listeners) = response.proplist_get_atom_key("listeners") {
            println!("\nListeners\n");
            if let Some(listener_list) = listeners.as_list() {
                if listener_list.is_empty() {
                    println!(" * (none)");
                } else {
                    for listener in listener_list {
                        if let Some(listener_tuple) = listener.as_tuple() {
                            if listener_tuple.len() == 2 {
                                let protocol = listener_tuple[0].atom_name().unwrap_or("unknown");
                                if let Some(port) = listener_tuple[1].as_integer() {
                                    println!(" * {} on port {}", protocol, port);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn product_info(node: &mut Node, target_node: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Getting product information from {}...", target_node);
    }

    let response = node
        .rpc_call(target_node, "rabbit", "product_info", vec![])
        .await
        .context("Failed to call rabbit:product_info/0")?;

    let response = unwrap_rpc_response(response)?;

    if quiet {
        println!("{:?}", response);
        return Ok(());
    }

    println!("\nProduct Information:");
    println!("{}", "=".repeat(40));

    if let OwnedTerm::Map(map) = response {
        for (key, value) in map {
            if let OwnedTerm::Atom(key_atom) = key {
                let value_str = if let Some(s) = value.as_erlang_string() {
                    s
                } else {
                    format!("{:?}", value)
                };
                println!("  {}: {}", key_atom, value_str);
            }
        }
    } else {
        println!("Unexpected response format: {:?}", response);
    }

    Ok(())
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

    let client_node_name = format!(
        "examples_diagnostics_{}@{}",
        std::process::id(),
        remote_host
    );

    let mut node = Node::new_hidden(client_node_name, cookie);
    node.start(0).await.context("Failed to start Erlang node")?;

    if !quiet {
        println!("Connecting to {}...", target_node);
    }

    node.connect(&target_node)
        .await
        .context("Failed to connect to RabbitMQ node")?;

    match matches.subcommand() {
        Some(("list_queues", sub_matches)) => {
            let vhost = sub_matches
                .get_one::<String>("vhost")
                .map(|s| s.to_string());
            let columns: Vec<String> = sub_matches
                .get_many::<String>("columns")
                .map(|vals| vals.map(|s| s.to_string()).collect())
                .unwrap_or_default();
            list_queues(&mut node, &target_node, vhost, columns, quiet).await
        }
        Some(("log_location", _)) => log_location(&mut node, &target_node, quiet).await,
        Some(("status", _)) => status(&mut node, &target_node, quiet).await,
        Some(("product_info", _)) => product_info(&mut node, &target_node, quiet).await,
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
