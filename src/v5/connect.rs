use crate::{
    QoS,
    common::coder::{Decoder, Encoder, read_mqtt_bytes, read_mqtt_string},
    error::ProtoError,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};

const PROTOCOL_NAME: &str = "MQTT";
const PROTOCOL_LEVEL: u8 = 5;


/**
 * MQTT v5 连接报文
 */
#[derive(Debug, Clone)]
pub struct Connect {
    // 固定头部字段
    clean_start: bool,
    keep_alive: u16,

    // 可变头部
    properties: Properties,
    client_id: String,

    // 遗嘱信息
    will: Option<LastWill>,
    // 认证信息
    auth: Option<Auth>,
}

#[derive(Debug, Clone)]
struct LastWill {
    topic: String,
    payload: Bytes,
    qos: QoS,
    retain: bool,
    properties: Properties,
}

#[derive(Debug, Clone)]
struct Auth {
    method: String,
    data: Bytes,
}

impl Connect {
    pub fn new(client_id: String, keep_alive: u16, clean_start: bool) -> Self {
        Self {
            clean_start,
            keep_alive,
            properties: Properties::default(),
            client_id,
            will: None,
            auth: None,
        }
    }

    pub fn with_will(mut self, will: LastWill) -> Self {
        self.will = Some(will);
        self
    }

    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_properties(mut self, properties: Properties) -> Self {
        self.properties = properties;
        self
    }
}

impl Encoder for Connect {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let start_pos = buffer.len();

        // 协议名
        buffer.put_u16(PROTOCOL_NAME.len() as u16);
        buffer.put_slice(PROTOCOL_NAME.as_bytes());

        // 协议版本
        buffer.put_u8(PROTOCOL_LEVEL);

        // 连接标志
        let mut flags = 0u8;
        flags |= (self.clean_start as u8) << 1; // 保持正确位移操作
        if let Some(will) = &self.will {
            flags |= 0b00000100; // Will Flag
            flags |= (will.qos as u8) << 3;
            flags |= (will.retain as u8) << 5;
        }
        if self.auth.is_some() {
            flags |= 0b10000000; // Password Flag
            flags |= 0b01000000; // Username Flag
        }
        buffer.put_u8(flags);

        // 保活时间
        buffer.put_u16(self.keep_alive);

        // 属性
        self.properties.encode(buffer)?;

        // 客户端ID
        buffer.put_u16(self.client_id.len() as u16);
        buffer.put_slice(self.client_id.as_bytes());

        // 遗嘱信息
        if let Some(will) = &self.will {
            // 遗嘱属性
            will.properties.encode(buffer)?;

            // 遗嘱主题
            buffer.put_u16(will.topic.len() as u16);
            buffer.put_slice(will.topic.as_bytes());

            // 遗嘱消息
            buffer.put_u32(will.payload.len() as u32);
            buffer.put_slice(&will.payload);
        }

        // 认证信息
        if let Some(auth) = &self.auth {
            buffer.put_u16(auth.method.len() as u16);
            buffer.put_slice(auth.method.as_bytes());

            buffer.put_u32(auth.data.len() as u32);
            buffer.put_slice(&auth.data);
        }

        Ok(buffer.len() - start_pos)
    }
}

impl Decoder for Connect {
    type Item = Connect;
    type Error = ProtoError;

    fn decode(mut bytes: Bytes) -> Result<Self, ProtoError> {
        // 校验协议名
        let protocol_name = read_mqtt_string(&mut bytes)?;
        if protocol_name != PROTOCOL_NAME {
            return Err(ProtoError::NotKnow);
        }

        // 校验协议版本
        let protocol_level = bytes.get_u8();
        if protocol_level != PROTOCOL_LEVEL {
            return Err(ProtoError::NotKnow);
        }

        // 解析标志位
        let flags = bytes.get_u8();
        let clean_start = (flags & 0b10000000) != 0;

        // 保活时间
        let keep_alive = bytes.get_u16();

        // 解析属性
        let properties = Properties::decode(bytes.clone())?;

        // 客户端ID
        let client_id = read_mqtt_string(&mut bytes)?;

        let mut connect =
            Connect::new(client_id, keep_alive, clean_start).with_properties(properties);

        // 解析遗嘱信息
        if flags & 0b00000100 != 0 {
            let will_properties = Properties::decode(bytes.clone())?;
            let topic = read_mqtt_string(&mut bytes)?;
            let payload_len = bytes.get_u32() as usize;
            let payload = bytes.split_to(payload_len);

            connect.will = Some(LastWill {
                topic,
                payload,
                qos: QoS::try_from((flags >> 3) & 0b11)?,
                retain: (flags & 0b00100000) != 0,
                properties: will_properties,
            });
        }

        // 解析认证信息
        if flags & 0b10000000 != 0 {
            let method = read_mqtt_string(&mut bytes)?;
            let data_len = bytes.get_u32() as usize;
            let data = bytes.split_to(data_len);

            connect.auth = Some(Auth { method, data });
        }

        Ok(connect)
    }
}
#[derive(Debug, Clone, Default)]
pub struct Properties {
    pub session_expiry_interval: Option<u32>,
    pub receive_maximum: Option<u16>,
    pub user_properties: Vec<(String, String)>,
    // 其他v5属性...
}

impl Encoder for Properties {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let mut total_len = 0;
        const MAX_PROPERTIES_LEN: usize = 65535;

        // Session Expiry Interval
        if let Some(expiry) = self.session_expiry_interval {
            buffer.put_u8(0x11);
            buffer.put_u32(expiry);
            total_len += 5;
        }

        // Receive Maximum
        if let Some(max) = self.receive_maximum {
            buffer.put_u8(0x12);
            buffer.put_u16(max);
            total_len += 3;
        }

        // User Properties
        for (key, value) in &self.user_properties {
            let entry_len = 1 + 2 + key.len() + 2 + value.len();
            if total_len + entry_len > MAX_PROPERTIES_LEN {
                return Err(ProtoError::OutOfMaxPropertySize);
            }
            buffer.put_u8(0x26);
            buffer.put_u16(key.len() as u16);
            buffer.put_slice(key.as_bytes());
            buffer.put_u16(value.len() as u16);
            buffer.put_slice(value.as_bytes());
            total_len += entry_len;
        }

        Ok(total_len)
    }
}

impl Decoder for Properties {
    type Item = Properties;
    type Error = ProtoError;

    fn decode(mut bytes: bytes::Bytes) -> Result<Self, ProtoError> {
        let mut properties = Properties::default();

        while bytes.has_remaining() {
            let property_id = bytes.get_u8();
            match property_id {
                0x11 => properties.session_expiry_interval = Some(bytes.get_u32()),
                0x12 => properties.receive_maximum = Some(bytes.get_u16()),
                0x26 => {
                    let key = read_mqtt_string(&mut bytes)?;
                    let value = read_mqtt_string(&mut bytes)?;
                    properties.user_properties.push((key, value));
                }
                _ => return Err(ProtoError::NotKnow),
            }
        }

        Ok(properties)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    //
    // #[test]
    // fn test_basic_connect_encoding() {
    //     let connect = Connect::new("test_client".to_string(), 60, true);
    //     let mut buffer = BytesMut::new();
    //     let encoded_len = connect.encode(&mut buffer).unwrap();
    //
    //     // 验证Clean Start标志位（修正位移错误）
    //     assert_eq!(buffer[3] & 0b00000010, 0b00000010); // 验证协议标志字节第1位
    // }
    //
    // #[test]
    // fn test_auth_method_data_roundtrip() {
    //     let auth = Auth {
    //         method: "SCRAM-SHA-256".to_string(),
    //         data: Bytes::from(vec![0x01, 0x02, 0x03]),
    //     };
    //
    //     let connect = Connect::new("auth_client".to_string(), 10, true).with_auth(auth);
    //
    //     let mut buffer = BytesMut::new();
    //     connect.encode(&mut buffer).unwrap();
    //     let bytes_copy = buffer.clone().freeze();
    //     let decoded = Connect::decode(bytes_copy).unwrap();
    //
    //     // 添加二进制数据直接对比
    //     let encoded_auth_data = &buffer[buffer.len() - 7..];
    //     assert_eq!(
    //         encoded_auth_data,
    //         &[
    //             0x00, 0x0D, 0x53, 0x43, 0x52, 0x41, 0x4D, 0x2D, 0x53, 0x48, 0x41, 0x2D, 0x32, 0x35,
    //             0x36, 0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03
    //         ]
    //     );
    // }
}
