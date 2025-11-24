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

use erltf_serde::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Resource {
    virtual_host: String,
    kind: String,
    name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct User {
    username: String,
    tags: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct InternalUser {
    username: String,
    password_hash: Vec<u8>,
    tags: Vec<String>,
    hashing_algorithm: String,
    limits: HashMap<String, i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Connection {
    name: String,
    peer_host: String,
    peer_port: u16,
    protocol: String,
    user: String,
    vhost: String,
    timeout_sec: u16,
    frame_max: u32,
    channel_max: u16,
    connected_at: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TrackedConnection {
    id: (String, String),
    node: String,
    vhost: String,
    name: String,
    protocol: String,
    peer_host: String,
    peer_port: u16,
    username: String,
    connected_at: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BasicProperties {
    content_type: Option<String>,
    content_encoding: Option<String>,
    headers: Option<HashMap<String, String>>,
    delivery_mode: Option<u8>,
    priority: Option<u8>,
    correlation_id: Option<String>,
    reply_to: Option<String>,
    expiration: Option<String>,
    message_id: Option<String>,
    timestamp: Option<i64>,
    user_id: Option<String>,
    app_id: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BasicMessage {
    exchange_name: Resource,
    routing_keys: Vec<String>,
    content: Vec<u8>,
    is_persistent: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct QueueDeclare {
    queue: String,
    passive: bool,
    durable: bool,
    exclusive: bool,
    auto_delete: bool,
    arguments: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ExchangeDeclare {
    exchange: String,
    exchange_type: String,
    passive: bool,
    durable: bool,
    auto_delete: bool,
    internal: bool,
    arguments: HashMap<String, i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Binding {
    source: String,
    key: String,
    destination: String,
    args: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BasicPublish {
    exchange: String,
    routing_key: String,
    mandatory: bool,
    immediate: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BasicDeliver {
    consumer_tag: String,
    delivery_tag: u64,
    redelivered: bool,
    exchange: String,
    routing_key: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BasicAck {
    delivery_tag: u64,
    multiple: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MqttPacketConnect {
    proto_ver: u8,
    clean_start: bool,
    keep_alive: u16,
    client_id: String,
    username: Option<String>,
    password: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MqttPacketPublish {
    topic_name: String,
    packet_id: Option<u16>,
    qos: u8,
    retain: bool,
    payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MqttSubscription {
    topic_filter: String,
    qos: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Amqp10Header {
    durable: Option<bool>,
    priority: Option<u8>,
    ttl: Option<u32>,
    first_acquirer: Option<bool>,
    delivery_count: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Amqp10Properties {
    message_id: Option<String>,
    user_id: Option<Vec<u8>>,
    to: Option<String>,
    subject: Option<String>,
    reply_to: Option<String>,
    correlation_id: Option<String>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    creation_time: Option<i64>,
    group_id: Option<String>,
}

//
// Resource and User Types
//

#[test]
fn test_resource_roundtrip() {
    let resource = Resource {
        virtual_host: "/".to_string(),
        kind: "queue".to_string(),
        name: "test_queue".to_string(),
    };

    let bytes = to_bytes(&resource).unwrap();
    let decoded: Resource = from_bytes(&bytes).unwrap();
    assert_eq!(resource, decoded);
}

#[test]
fn test_user_roundtrip() {
    let user = User {
        username: "guest".to_string(),
        tags: vec!["administrator".to_string(), "monitoring".to_string()],
    };

    let bytes = to_bytes(&user).unwrap();
    let decoded: User = from_bytes(&bytes).unwrap();
    assert_eq!(user, decoded);
}

#[test]
fn test_internal_user_with_limits() {
    let mut limits = HashMap::new();
    limits.insert("max-connections".to_string(), 100);
    limits.insert("max-channels".to_string(), 1000);

    let user = InternalUser {
        username: "admin".to_string(),
        password_hash: vec![0xDE, 0xAD, 0xBE, 0xEF],
        tags: vec!["administrator".to_string()],
        hashing_algorithm: "rabbit_password_hashing_sha256".to_string(),
        limits,
    };

    let bytes = to_bytes(&user).unwrap();
    let decoded: InternalUser = from_bytes(&bytes).unwrap();
    assert_eq!(user, decoded);
}

//
// Connection Types
//

#[test]
fn test_connection_metadata() {
    let connection = Connection {
        name: "127.0.0.1:55054 -> 127.0.0.1:5672".to_string(),
        peer_host: "127.0.0.1".to_string(),
        peer_port: 55054,
        protocol: "AMQP 0-9-1".to_string(),
        user: "guest".to_string(),
        vhost: "/".to_string(),
        timeout_sec: 60,
        frame_max: 131072,
        channel_max: 2047,
        connected_at: 1609459200,
    };

    let bytes = to_bytes(&connection).unwrap();
    let decoded: Connection = from_bytes(&bytes).unwrap();
    assert_eq!(connection, decoded);
}

#[test]
fn test_tracked_connection() {
    let tracked = TrackedConnection {
        id: ("rabbit@localhost".to_string(), "conn123".to_string()),
        node: "rabbit@localhost".to_string(),
        vhost: "/".to_string(),
        name: "127.0.0.1:55054".to_string(),
        protocol: "AMQP 0-9-1".to_string(),
        peer_host: "127.0.0.1".to_string(),
        peer_port: 55054,
        username: "guest".to_string(),
        connected_at: 1609459200,
    };

    let bytes = to_bytes(&tracked).unwrap();
    let decoded: TrackedConnection = from_bytes(&bytes).unwrap();
    assert_eq!(tracked, decoded);
}

//
// AMQP 0-9-1 Message Properties
//

#[test]
fn test_basic_properties_full() {
    let mut headers = HashMap::new();
    headers.insert("x-custom".to_string(), "value".to_string());

    let props = BasicProperties {
        content_type: Some("application/json".to_string()),
        content_encoding: Some("utf-8".to_string()),
        headers: Some(headers),
        delivery_mode: Some(2),
        priority: Some(5),
        correlation_id: Some("abc123".to_string()),
        reply_to: Some("reply_queue".to_string()),
        expiration: Some("60000".to_string()),
        message_id: Some("msg-001".to_string()),
        timestamp: Some(1609459200),
        user_id: Some("guest".to_string()),
        app_id: Some("my_app".to_string()),
    };

    let bytes = to_bytes(&props).unwrap();
    let decoded: BasicProperties = from_bytes(&bytes).unwrap();
    assert_eq!(props, decoded);
}

#[test]
fn test_basic_properties_minimal() {
    let props = BasicProperties {
        content_type: None,
        content_encoding: None,
        headers: None,
        delivery_mode: Some(1),
        priority: None,
        correlation_id: None,
        reply_to: None,
        expiration: None,
        message_id: None,
        timestamp: None,
        user_id: None,
        app_id: None,
    };

    let bytes = to_bytes(&props).unwrap();
    let decoded: BasicProperties = from_bytes(&bytes).unwrap();
    assert_eq!(props, decoded);
}

//
// AMQP 0-9-1 Messages
//

#[test]
fn test_basic_message() {
    let message = BasicMessage {
        exchange_name: Resource {
            virtual_host: "/".to_string(),
            kind: "exchange".to_string(),
            name: "amq.topic".to_string(),
        },
        routing_keys: vec!["events.user.created".to_string()],
        content: b"message payload".to_vec(),
        is_persistent: true,
    };

    let bytes = to_bytes(&message).unwrap();
    let decoded: BasicMessage = from_bytes(&bytes).unwrap();
    assert_eq!(message, decoded);
}

//
// AMQP 0-9-1 Queue Operations
//

#[test]
fn test_queue_declare_durable() {
    let mut args = HashMap::new();
    args.insert("x-max-length".to_string(), "10000".to_string());
    args.insert("x-message-ttl".to_string(), "3600000".to_string());

    let declare = QueueDeclare {
        queue: "my_queue".to_string(),
        passive: false,
        durable: true,
        exclusive: false,
        auto_delete: false,
        arguments: args,
    };

    let bytes = to_bytes(&declare).unwrap();
    let decoded: QueueDeclare = from_bytes(&bytes).unwrap();
    assert_eq!(declare, decoded);
}

#[test]
fn test_queue_declare_exclusive() {
    let declare = QueueDeclare {
        queue: "".to_string(),
        passive: false,
        durable: false,
        exclusive: true,
        auto_delete: true,
        arguments: HashMap::new(),
    };

    let bytes = to_bytes(&declare).unwrap();
    let decoded: QueueDeclare = from_bytes(&bytes).unwrap();
    assert_eq!(declare, decoded);
}

//
// AMQP 0-9-1 Exchange Operations
//

#[test]
fn test_exchange_declare_topic() {
    let mut args = HashMap::new();
    args.insert("alternate-exchange".to_string(), 1);

    let declare = ExchangeDeclare {
        exchange: "events".to_string(),
        exchange_type: "topic".to_string(),
        passive: false,
        durable: true,
        auto_delete: false,
        internal: false,
        arguments: args,
    };

    let bytes = to_bytes(&declare).unwrap();
    let decoded: ExchangeDeclare = from_bytes(&bytes).unwrap();
    assert_eq!(declare, decoded);
}

//
// AMQP 0-9-1 Bindings
//

#[test]
fn test_binding() {
    let binding = Binding {
        source: "events".to_string(),
        key: "user.#".to_string(),
        destination: "user_events_queue".to_string(),
        args: vec![("x-match".to_string(), "all".to_string())],
    };

    let bytes = to_bytes(&binding).unwrap();
    let decoded: Binding = from_bytes(&bytes).unwrap();
    assert_eq!(binding, decoded);
}

//
// AMQP 0-9-1 Basic Class Operations
//

#[test]
fn test_basic_publish() {
    let publish = BasicPublish {
        exchange: "amq.topic".to_string(),
        routing_key: "events.order.created".to_string(),
        mandatory: true,
        immediate: false,
    };

    let bytes = to_bytes(&publish).unwrap();
    let decoded: BasicPublish = from_bytes(&bytes).unwrap();
    assert_eq!(publish, decoded);
}

#[test]
fn test_basic_deliver() {
    let deliver = BasicDeliver {
        consumer_tag: "ctag-001".to_string(),
        delivery_tag: 12345,
        redelivered: false,
        exchange: "amq.topic".to_string(),
        routing_key: "events.user.login".to_string(),
    };

    let bytes = to_bytes(&deliver).unwrap();
    let decoded: BasicDeliver = from_bytes(&bytes).unwrap();
    assert_eq!(deliver, decoded);
}

#[test]
fn test_basic_ack_single() {
    let ack = BasicAck {
        delivery_tag: 100,
        multiple: false,
    };

    let bytes = to_bytes(&ack).unwrap();
    let decoded: BasicAck = from_bytes(&bytes).unwrap();
    assert_eq!(ack, decoded);
}

#[test]
fn test_basic_ack_multiple() {
    let ack = BasicAck {
        delivery_tag: 500,
        multiple: true,
    };

    let bytes = to_bytes(&ack).unwrap();
    let decoded: BasicAck = from_bytes(&bytes).unwrap();
    assert_eq!(ack, decoded);
}

//
// MQTT Protocol Types
//

#[test]
fn test_mqtt_connect() {
    let connect = MqttPacketConnect {
        proto_ver: 4,
        clean_start: true,
        keep_alive: 60,
        client_id: "mqtt_client_001".to_string(),
        username: Some("mqtt_user".to_string()),
        password: Some(b"secret".to_vec()),
    };

    let bytes = to_bytes(&connect).unwrap();
    let decoded: MqttPacketConnect = from_bytes(&bytes).unwrap();
    assert_eq!(connect, decoded);
}

#[test]
fn test_mqtt_connect_anonymous() {
    let connect = MqttPacketConnect {
        proto_ver: 5,
        clean_start: false,
        keep_alive: 120,
        client_id: "anonymous_client".to_string(),
        username: None,
        password: None,
    };

    let bytes = to_bytes(&connect).unwrap();
    let decoded: MqttPacketConnect = from_bytes(&bytes).unwrap();
    assert_eq!(connect, decoded);
}

#[test]
fn test_mqtt_publish_qos0() {
    let publish = MqttPacketPublish {
        topic_name: "sensor/temperature".to_string(),
        packet_id: None,
        qos: 0,
        retain: false,
        payload: b"22.5".to_vec(),
    };

    let bytes = to_bytes(&publish).unwrap();
    let decoded: MqttPacketPublish = from_bytes(&bytes).unwrap();
    assert_eq!(publish, decoded);
}

#[test]
fn test_mqtt_publish_qos1_retained() {
    let publish = MqttPacketPublish {
        topic_name: "device/status".to_string(),
        packet_id: Some(42),
        qos: 1,
        retain: true,
        payload: b"online".to_vec(),
    };

    let bytes = to_bytes(&publish).unwrap();
    let decoded: MqttPacketPublish = from_bytes(&bytes).unwrap();
    assert_eq!(publish, decoded);
}

#[test]
fn test_mqtt_subscription() {
    let sub = MqttSubscription {
        topic_filter: "home/+/temperature".to_string(),
        qos: 1,
    };

    let bytes = to_bytes(&sub).unwrap();
    let decoded: MqttSubscription = from_bytes(&bytes).unwrap();
    assert_eq!(sub, decoded);
}

//
// AMQP 1.0 Types
//

#[test]
fn test_amqp10_header() {
    let header = Amqp10Header {
        durable: Some(true),
        priority: Some(5),
        ttl: Some(60000),
        first_acquirer: Some(false),
        delivery_count: Some(0),
    };

    let bytes = to_bytes(&header).unwrap();
    let decoded: Amqp10Header = from_bytes(&bytes).unwrap();
    assert_eq!(header, decoded);
}

#[test]
fn test_amqp10_properties() {
    let props = Amqp10Properties {
        message_id: Some("msg-12345".to_string()),
        user_id: Some(b"user01".to_vec()),
        to: Some("/queue/orders".to_string()),
        subject: Some("new_order".to_string()),
        reply_to: Some("/queue/replies".to_string()),
        correlation_id: Some("corr-001".to_string()),
        content_type: Some("application/json".to_string()),
        content_encoding: Some("utf-8".to_string()),
        creation_time: Some(1609459200),
        group_id: Some("batch-001".to_string()),
    };

    let bytes = to_bytes(&props).unwrap();
    let decoded: Amqp10Properties = from_bytes(&bytes).unwrap();
    assert_eq!(props, decoded);
}

//
// Complex Nested Structures
//

#[test]
fn test_complex_nested_structure() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct ComplexMessage {
        connection: Connection,
        properties: BasicProperties,
        queue_declare: QueueDeclare,
        bindings: Vec<Binding>,
    }

    let msg = ComplexMessage {
        connection: Connection {
            name: "conn1".to_string(),
            peer_host: "10.0.0.1".to_string(),
            peer_port: 5672,
            protocol: "AMQP".to_string(),
            user: "test".to_string(),
            vhost: "/test".to_string(),
            timeout_sec: 30,
            frame_max: 131072,
            channel_max: 255,
            connected_at: 1000,
        },
        properties: BasicProperties {
            content_type: Some("text/plain".to_string()),
            content_encoding: None,
            headers: None,
            delivery_mode: Some(2),
            priority: None,
            correlation_id: None,
            reply_to: None,
            expiration: None,
            message_id: None,
            timestamp: None,
            user_id: None,
            app_id: None,
        },
        queue_declare: QueueDeclare {
            queue: "test".to_string(),
            passive: false,
            durable: true,
            exclusive: false,
            auto_delete: false,
            arguments: HashMap::new(),
        },
        bindings: vec![
            Binding {
                source: "ex1".to_string(),
                key: "key1".to_string(),
                destination: "q1".to_string(),
                args: vec![],
            },
            Binding {
                source: "ex2".to_string(),
                key: "key2".to_string(),
                destination: "q2".to_string(),
                args: vec![],
            },
        ],
    };

    let bytes = to_bytes(&msg).unwrap();
    let decoded: ComplexMessage = from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}
