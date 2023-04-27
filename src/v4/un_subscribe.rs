use bytes::{Buf, Bytes, BytesMut};

use crate::{error::ProtoError, v4::VariableDecoder};

use super::{
    decoder::{self, write_mqtt_string},
    fixed_header::FixedHeader,
    Decoder, Encoder, GeneralVariableHeader,
};


/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 1   | 0   | 1   | 1   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 0   | 1   | 0   |
/// | byte3 | 报   | 文   | 标  | 识   | 符  | M   | S   | B   |
/// | byte4 | 报   | 文   | 标  | 识   | 符  | L   | S   | B   |

#[derive(Debug, Clone)]
pub struct UnSubscribe {
    fixed_header: FixedHeader,
    variable_header: GeneralVariableHeader,
    topices: Vec<String>,
}

impl UnSubscribe {
    pub fn new(
        fixed_header: FixedHeader,
        variable_header: GeneralVariableHeader,
        topices: Vec<String>,
    ) -> Self {
        Self {
            fixed_header,
            variable_header,
            topices,
        }
    }

    pub fn message_id(&self) -> usize {
        self.variable_header.message_id
    }

    pub fn topices(&self) -> Vec<String> {
        self.topices.clone()
    }
}

impl Encoder for UnSubscribe {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let resp = self.fixed_header.encode(buffer);
        match resp {
            Ok(len) => {
                if let Ok(v_len) = self.variable_header.encode(buffer) {
                    let resp = len + v_len;
                    let mut topics_len = 0;
                    for temp in &self.topices {
                        write_mqtt_string(buffer, temp);
                        let topic_len = temp.len() + 2;
                        topics_len += topic_len;
                    }
                    return Ok(resp + topics_len);
                }
                Err(ProtoError::NotKnow)
            }
            Err(err) => Err(err),
        }
    }
}

impl Decoder for UnSubscribe {
    type Item = UnSubscribe;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = decoder::read_fixed_header(&mut bytes);
        // println!("resp: {:?}", resp);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                if let Ok(variable_header) = GeneralVariableHeader::decode(&mut bytes) {
                    let mut topices = Vec::new();
                    // println!("bytes: {:?}", bytes);
                    while !bytes.is_empty() {
                        let topic = decoder::read_mqtt_string(&mut bytes);
                        match topic {
                            Ok(topic) => topices.push(topic),
                            Err(e) => return Err(e),
                        }
                    }
                    return Ok(UnSubscribe::new(fixed_header, variable_header, topices));
                }
                Err(ProtoError::DecodeGeneralVariableHeaderError)
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::v4::{builder::MqttMessageBuilder, Decoder, Encoder};

    use super::UnSubscribe;

    fn build_sub() -> UnSubscribe {
        let mut topices = Vec::new();
        let topic1 = "/test".to_string();
        let topic2 = "/name".to_string();
        topices.push(topic1);
        topices.push(topic2);
        // Subscribe::new(topices, 1892).unwrap()
        let unsub = MqttMessageBuilder::unsubscriber()
            .message_id(65531)
            .topices(topices)
            .build()
            .unwrap();
        unsub
    }

    #[test]
    fn encode_and_decode_subscribe_shoud_be_work() {
        let sub = build_sub();
        println!("原始sub = {:?}", sub);
        let mut bytes = BytesMut::new();
        sub.encode(&mut bytes);
        println!("buffer = {:?}", bytes);
        let resp = UnSubscribe::decode(bytes.into());
        match resp {
            Ok(sub) => println!("新的sub = {:?}", sub),
            Err(e) => println!("解码异常 {}", e),
        }
    }
}
