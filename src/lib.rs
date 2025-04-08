/*! 一个Rust实现的mqtt协议解析库

```rust
   use bytes::Bytes;
   use walle_mqtt_protocol::{MqttVersion, QoS};
   use walle_mqtt_protocol::v4::builder::MqttMessageBuilder;
   let connect = MqttMessageBuilder::connect()
           .client_id("client_01")
           .keep_alive(10)
           .clean_session(true)
           .username("rump")
           .password("mq")
           .protocol_level(MqttVersion::V4)
           .retain(false)
           .will_qos(QoS::AtLeastOnce)
           .will_topic("/a")
           .will_message(Bytes::from_static(b"offline"))
           .build().unwrap();
 ```

*/

use error::ProtoError;
use serde::{Deserialize, Serialize};
pub mod common;
pub mod error;
pub mod v4;
pub mod v5;

/// MQTT报文中protocol name字段
pub const PROTOCOL_NAME: &'static str = "MQTT";

/// mqtt协议不同的版本，这里取最常用的两个版本
#[derive(Debug, Clone, PartialEq)]
pub enum MqttVersion {
    V4,
    V5,
}

/// 数据类型
#[derive(Debug, Clone, PartialEq, Default, PartialOrd)]
pub enum MessageType {
    #[default]
    CONNECT,
    CONNACK,
    PUBLISH,
    PUBACK,
    PUBREL,
    PUBREC,
    PUBCOMP,
    PINGREQ,
    PINGRESP,
    SUBSCRIBE,
    SUBACK,
    UNSUBSCRIBE,
    UNSUBACK,
    DISCONNECT,
}

/////////////////////////////////////////////////////////////////////////
/// mqtt协议中对消息质量的定义
/// mqtt消息质量分为三种：
/// - AtMostOnce：使用0表示
/// - AtLeastOnce：使用1表示
/// - ExactlyOnce：使用2表示
/////////////////////////////////////////////////////////////////////////
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum QoS {
    // 最多
    #[default]
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl From<QoS> for u8 {
    fn from(value: QoS) -> Self {
        match value {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 1,
            QoS::ExactlyOnce => 2,
        }
    }
}

impl TryFrom<u8> for QoS {
    type Error = ProtoError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            n => Err(ProtoError::QoSError(n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::v4::builder::MqttMessageBuilder;

    #[test]
    fn test() {
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
        println!("connect = {:?}", connect);
    }
}
