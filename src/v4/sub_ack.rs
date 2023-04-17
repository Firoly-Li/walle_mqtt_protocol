use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{error::ProtoError, QoS};

use super::{
    decoder::{self},
    fixed_header::FixedHeader,
    Decoder, Encoder, GeneralVariableHeader, VariableDecoder,
};

/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 1   | 0   | 0   | 1   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 1   | 0   | 1   |
/// | byte3 | 报   | 文   | 标  | 识   | 符  | M   | S   | B   |
/// | byte4 | 报   | 文   | 标  | 识   | 符  | L   | S   | B   |
/// | byte5 | x   | 0   | 0  | 0   | 0  | 0   | x   | x   |
#[derive(Debug)]
pub struct SubAck {
    fixed_header: FixedHeader,
    variable_header: GeneralVariableHeader,
    acks: Vec<u8>,
}

impl SubAck {
    pub fn new(
        mut fixed_header: FixedHeader,
        variable_header: GeneralVariableHeader,
        acks: Vec<u8>,
    ) -> Self {
        fixed_header.set_remaining_length(acks.len());
        Self {
            fixed_header,
            variable_header,
            acks,
        }
    }

    pub fn message_id(&self) -> usize {
        self.variable_header.message_id
    }

    pub fn qos(&self) -> Option<QoS> {
        self.fixed_header.qos()
    }
}

//////////////////////////////////////////////////////////
/// 为SubAck实现Encoder trait
/////////////////////////////////////////////////////////
impl Encoder for SubAck {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let resp = self.fixed_header.encode(buffer);
        match resp {
            Ok(fixed_header_len) => {
                let resp = self.variable_header.encode(buffer);
                match resp {
                    Ok(variable_header_len) => {
                        //todo acks还未进行encode
                        let acks = self.acks.iter();
                        for ack in acks {
                            buffer.put_u8(ack.clone());
                        }
                        Ok(fixed_header_len + variable_header_len + self.acks.len())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Decoder for SubAck {
    type Item = SubAck;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        // 读取fixed_header
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = GeneralVariableHeader::decode(&mut bytes);
                match resp {
                    Ok(variable_header) => {
                        let acks: Vec<u8> = Vec::from(bytes);
                        Ok(SubAck::new(fixed_header, variable_header, acks))
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::{
        v4::{builder::MqttMessageBuilder, Decoder, Encoder},
        QoS,
    };

    use super::SubAck;

    #[test]
    fn test() {
        let resp = MqttMessageBuilder::sub_ack()
            .message_id(12)
            .acks(vec![0, 1, 2, 1, 1, 0])
            .build()
            .unwrap();
        println!("原始的sub = {:?}", resp);
        let mut bytes = BytesMut::new();
        resp.encode(&mut bytes);

        let resp = SubAck::decode(bytes.into());
        match resp {
            Ok(sub) => println!("新的sub = {:?}", sub),
            Err(e) => println!("解码异常 {}", e),
        }
    }
}
