# walle_mqtt_protocol

一个Rust实现的mqtt协议解析库

```rust
let connect = MqttMessageBuilder::connect()
        .client_id("client_01")
        .keep_alive(10)
        .clean_session(true)
        .username("rump")
        .password("mq")
        .protocol_level(crate::MqttVersion::V4)
        .retain(false)
        .will_qos(crate::QoS::AtLeastOnce)
        .will_topic("/a")
        .will_message(Bytes::from_static(b"offline"))
        .build();
```
MqttMessageBuilder: 是MQTT报文的构建器，提供了多种MQTT报文的构建方式。