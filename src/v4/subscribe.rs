use super::{
    decoder, fixed_header::FixedHeader, Decoder, Encoder, GeneralVariableHeader, VariableDecoder,
};
use crate::{error::ProtoError, Topic};
use bytes::{Buf, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct Subscribe {
    // 固定报头
    fixed_header: FixedHeader,
    // 可变报头，里面包含消息id,可变报头占用2字节
    variable_header: GeneralVariableHeader,
    // payload中包括了各种订阅的topic，格式为：topic_len|topic|qos,
    // 所以Topic的长度是，2+topic_len+1 = 3+topic_len
    topices: Vec<Topic>,
}

impl Subscribe {
    pub fn new(
        fixed_header: FixedHeader,
        variable_header: GeneralVariableHeader,
        topices: Vec<Topic>,
    ) -> Self {
        Self {
            fixed_header,
            variable_header,
            topices,
        }
        .build()
    }

    fn topics_len(&self) -> usize {
        let mut len = 0;
        for temp in &self.topices {
            len += temp.name_len() + 3;
        }
        len
    }

    pub fn fixed_header(&self) -> FixedHeader {
        self.fixed_header.clone()
    }

    pub fn variable_header(&self) -> GeneralVariableHeader {
        self.variable_header.clone()
    }

    pub fn topices(&self) -> Vec<Topic> {
        self.topices.clone()
    }

    fn build(mut self) -> Self {
        let topic_len = self.topics_len();
        let remaining_len = topic_len + 2;
        self.fixed_header.set_remaining_length(remaining_len);
        self
    }
}

//////////////////////////////////////////////////////
/// 为Subscribe实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for Subscribe {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let resp = self.fixed_header.encode(buffer);
        match resp {
            Ok(len) => {
                if let Ok(v_len) = self.variable_header.encode(buffer) {
                    let resp = len + v_len;
                    for temp in &self.topices {
                        temp.encode(buffer);
                    }
                    let topic_len = self.topics_len();
                    return Ok(resp + topic_len);
                }
                Err(ProtoError::NotKnow)
            }
            Err(err) => Err(err),
        }
    }
}

impl Decoder for Subscribe {
    type Item = Subscribe;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = decoder::read_fixed_header(&mut bytes);
        // println!("resp: {:?}", resp);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                if let Ok(variable_header) = GeneralVariableHeader::decode(&mut bytes) {
                    println!("bytes: {:?}", bytes);
                    let topices = Topic::read_topics(&mut bytes);
                    println!("topices: {:?}", topices);
                    match topices {
                        Ok(topices) => {
                            return Ok(Subscribe {
                                fixed_header,
                                variable_header,
                                topices,
                            });
                        }
                        Err(err) => return Err(err),
                    }
                }
                Err(ProtoError::NotKnow)
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::{
        v4::{builder::MqttMessageBuilder, Decoder, Encoder},
        Topic,
    };

    use super::Subscribe;

    fn build_sub() -> Subscribe {
        let mut topices = Vec::new();
        let topic1 = Topic::new("/name".to_string(), crate::QoS::AtLeastOnce);
        let topic2 = Topic::new("/test".to_string(), crate::QoS::AtMostOnce);
        topices.push(topic1);
        topices.push(topic2);
        // Subscribe::new(topices, 1892).unwrap()
        MqttMessageBuilder::subscribe()
            .topics(topices)
            .message_id(1892)
            .build()
            .unwrap()
    }

    #[test]
    fn encode_and_decode_subscribe_shoud_be_work() {
        let sub = build_sub();
        println!("原始sub = {:?}", sub);
        let mut bytes = BytesMut::new();
        sub.encode(&mut bytes);

        let resp = Subscribe::decode(bytes.into());
        match resp {
            Ok(sub) => println!("新的sub = {:?}", sub),
            Err(e) => println!("解码异常 {}", e),
        }
    }
}
