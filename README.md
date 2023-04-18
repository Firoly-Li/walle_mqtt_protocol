# walle_mqtt_protocol

## 快速开始
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
        .build().unwarp();
```
MqttMessageBuilder: 是MQTT报文的构建器，提供了多种MQTT报文的构建方式。
MqttMessageBuilder并没有具体的属性，MqttMessageBuilder提供了其他具体Builder的方法,例如：
```rust
pub fn connect() -> ConnectBuilder {
    ConnectBuilder::new()
}
```

## Encoder和Decoder
```rust
/// 编码
pub trait Encoder: Sync + Send + 'static {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError>;
}

/// 解码
pub trait Decoder: Sync + Send + 'static {
    // 定义的返回类型
    type Item;
    type Error;
    // 将bytes解析为对应的报文
    fn decode(bytes: Bytes) -> Result<Self::Item, Self::Error>;
}
```
Encoder和Decoder实现了Bytes和Mqtt报文的相互转换,每种MQTT报文类型都实现了Encoder和Decoder，以Connect报文为例：
```rust
fn encode_and_decode_for_connect_should_be_work() {
    let connect = build_connect().unwrap();
    println!("原始connect = {:?}", connect);
    let mut bytes = BytesMut::new();
    let mut bytes1 = BytesMut::new();
    // encoder
    connect.encode(&mut bytes).unwrap();
    println!("bytes = {:?}", bytes);
    let connect1 = Connect::decode(bytes.freeze());
    match connect1 {
        Ok(connect) => {
            println!("connect1 = {:?}", connect);
            connect.encode(&mut bytes1).unwrap();
            println!(
                "encode_and_decode_for_connect_shoud_be_work bytes1  = {:?}",
                bytes1
            );
            // decoder
            let connect2 = Connect::decode(bytes1into()).unwrap();
            println!("connect2 = {:?}", connect2);
        }
        Err(err) => println!("编解码出错"),
    }
}
```