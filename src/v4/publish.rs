use std::sync::Arc;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::ProtoError;

use super::{
    decoder::{self, read_mqtt_string, read_u16},
    fixed_header::FixedHeader,
    Decoder, Encoder, VariableDecoder,
};

pub const ONE_BYTE_MAX_LEN: usize = 127;
pub const TWO_BYTE_MAX_LEN: usize = 16383;
pub const THREE_BYTE_MAX_LEN: usize = 2097151;
pub const FOUR_BYTE_MAX_LEN: usize = 268435455;

/// 一个完整的publish报文的格式如下：
///
/// | 字节 | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   | 数值 | 备注     |
/// | ---- | --- | --- | --- | --- | --- | --- | --- | --- | ---- | -------- |
/// | 1    | 0   | 0   | 1   | 1   | 0   | 0   | 1   | 0   | 50   | 首字节   |
/// | 2    | 0   | 0   | 1   | 0   | 0   | 0   | 1   | 1   | 19   | 剩余长度 |
/// | 3    | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0    |          |
/// | 4    | 0   | 0   | 0   | 0   | 0   | 1   | 0   | 1   | 5    |          |
/// | 5    | 0   | 0   | 1   | 0   | 1   | 1   | 1   | 1   | 47   | /        |
/// | 6    | 0   | 1   | 1   | 1   | 0   | 1   | 0   | 0   | 116  | t        |
/// | 7    | 0   | 1   | 1   | 0   | 0   | 1   | 0   | 1   | 101  | e        |
/// | 8    | 0   | 1   | 1   | 1   | 0   | 0   | 1   | 1   | 115  | s        |
/// | 9    | 0   | 1   | 1   | 1   | 0   | 1   | 0   | 0   | 116  | t        |
/// | 10   | 0   | 1   | 1   | 1   | 1   | 0   | 1   | 0   | 122  |          |
/// | 11   | 0   | 1   | 1   | 1   | 0   | 0   | 0   | 0   | 112  |          |
/// | 12   | 0   | 0   | 1   | 1   | 0   | 0   | 0   | 1   | 49   | 1        |
/// | 13   | 0   | 0   | 1   | 1   | 0   | 0   | 1   | 0   | 50   | 2        |
/// | 14   | 0   | 0   | 1   | 1   | 0   | 0   | 1   | 1   | 51   | 3        |
/// | 15   | 0   | 0   | 1   | 1   | 0   | 1   | 0   | 0   | 52   | 4        |
/// | 16   | 0   | 0   | 1   | 1   | 0   | 1   | 0   | 1   | 53   | 5        |
/// | 17   | 0   | 0   | 1   | 1   | 0   | 1   | 1   | 0   | 54   | 6        |
/// | 18   | 0   | 0   | 1   | 1   | 0   | 1   | 1   | 1   | 55   | 7        |
/// | 19   | 0   | 0   | 1   | 1   | 1   | 0   | 0   | 0   | 56   | 8        |
/// | 20   | 0   | 0   | 1   | 1   | 1   | 0   | 0   | 1   | 57   | 9        |
/// | 21   | 0   | 0   | 1   | 1   | 0   | 0   | 0   | 0   | 48   | 0        |
#[derive(Debug, Clone)]
pub struct Publish {
    // 固定报头
    fixed_header: FixedHeader,
    // 可变报头
    variable_header: PublishVariableHeader,
    // payload 有效载荷
    payload: Bytes,
}

impl Publish {
    pub fn new(
        fixed_header: FixedHeader,
        variable_header: PublishVariableHeader,
        payload: Bytes,
    ) -> Self {
        Self {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn fixed_header(&self) -> FixedHeader {
        self.fixed_header.clone()
    }

    pub fn variable_header(&self) -> PublishVariableHeader {
        self.variable_header.clone()
    }

    pub fn payload(&self) -> Bytes {
        self.payload.clone()
    }

    pub fn update(mut self,message_id: usize) -> Self {
        let fixed_header = self.fixed_header;
        let variable_header = self.variable_header.update_message_id(message_id);
        let payload = self.payload;
        Self {
            fixed_header,
            variable_header,
            payload,
        }
    }
}

//////////////////////////////////////////////////////////
/// 为Publish实现Encoder trait
/////////////////////////////////////////////////////////
impl Encoder for Publish {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let resp = self.fixed_header.encode(buffer);
        match resp {
            Ok(fixed_header_len) => {
                let resp = self.variable_header.encode(buffer);
                match resp {
                    Ok(variable_header_len) => {
                        buffer.put(self.payload());
                        let resp = fixed_header_len + variable_header_len + self.payload().len();
                        Ok(resp)
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
}

//////////////////////////////////////////////////////////
/// 为Publish实现Decoder trait
/////////////////////////////////////////////////////////
impl Decoder for Publish {
    type Item = Publish;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        // 读取fixed_header
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = PublishVariableHeader::decode(&mut bytes);
                match resp {
                    Ok(variable_header) => Ok(Publish {
                        fixed_header,
                        variable_header,
                        payload: bytes,
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

//////////////////////////////////////////////
/// PublishVariableHeader
/////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct PublishVariableHeader {
    // variable_header的长度
    variable_header_len: usize,
    // topic
    topic: String,
    // message_id
    message_id: Option<usize>,
}
impl PublishVariableHeader {
    pub fn new(topic: String, message_id: Option<usize>) -> Self {
        Self {
            variable_header_len: Self::variable_len(topic.as_str()),
            topic,
            message_id,
        }
    }

    fn variable_len(topic: &str) -> usize {
        topic.len() + 4
    }
    pub fn variable_header_len(&self) -> usize {
        self.variable_header_len
    }
    pub fn topic(&self) -> String {
        self.topic.clone()
    }
    pub fn message_id(&self) -> Option<usize> {
        self.message_id
    }
    pub fn update_message_id(mut self,message_id: usize) -> Self {
        self.message_id = Some(message_id);
        self
    }
}

//////////////////////////////////////////////////////////
/// 为PublishVariableHeader实现VariableDecode trait
/////////////////////////////////////////////////////////
impl VariableDecoder for PublishVariableHeader {
    type Item = PublishVariableHeader;

    fn decode(bytes: &mut Bytes) -> Result<Self::Item, ProtoError> {
        let topic_resp = read_mqtt_string(bytes);
        let message_id_resp = read_u16(bytes);
        if let (Ok(topic), Ok(message_id)) = (topic_resp, message_id_resp) {
            return Ok(PublishVariableHeader::new(topic, Some(message_id.into())));
        }
        Err(ProtoError::NotKnow)
    }
}

//////////////////////////////////////////////////////////
/// 为PublishVariableHeader实现Encoder trait
/////////////////////////////////////////////////////////
impl Encoder for PublishVariableHeader {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let topic_len = self.topic.len();
        buffer.put_u16(topic_len as u16);
        let topic = self.topic.clone();
        buffer.put(topic.as_bytes());
        if let Some(message_id) = self.message_id {
            buffer.put_u16(message_id as u16);
        }
        Ok(self.variable_header_len)
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use crate::v4::{builder::MqttMessageBuilder, publish::Publish, Decoder, Encoder};

    #[test]
    fn publish_to_bytes() {
        let builder = MqttMessageBuilder::publish()
            .dup(false)
            .message_id(1)
            .qos(crate::QoS::AtLeastOnce)
            .retain(false)
            .topic("/test")
            .payload_str("hello world !")
            .build();
        match builder {
            Ok(publish) => {
                println!("publish = {:?}", publish);
                let fixed_header_len = publish.fixed_header.len();
                let variable_len = publish.variable_header.variable_header_len();
                let p_len = publish.payload.len();
                let r_len = publish.fixed_header.remaining_length();
                println!("remaining_len = {}", r_len);
                println!(
                    "fixed_header_len = {},variable_header_len = {},payload_len = {}",
                    fixed_header_len, variable_len, p_len
                );
                let mut buffer = BytesMut::new();
                publish.encode(&mut buffer);

                println!("buffer = {:?}", buffer);

                let publish = Publish::decode(buffer.freeze());
                match publish {
                    Ok(p) => {
                        println!("publish = {:?}", p);
                    }
                    Err(e) => {}
                }
            }
            Err(e) => {}
        }
    }
}
