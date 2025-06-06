use super::{
    GeneralVariableHeader, VariableDecoder,
    decoder::{self},
    fixed_header::FixedHeader,
};
use crate::common::coder::{Decoder, Encoder};
use crate::{QoS, error::ProtoError};
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// 订阅确认
/// SUBACK报文，反应了broker对client的SUBSCRIBE报文的回应，由于SUBSCRIBE报文可以同事订阅多个Topic，
/// 所以SUBACK需要对每个Topic均作出回应，其顺序是按照SUBACRIBE报文中Topic的顺序排列。每个Topic的返回码
/// 占用一个字节，允许的返回码如下：
///
/// | 字节 |        描述            | 7|6 |5 |4 |3 |2 |1 |0 |
/// |-----|------------------------|--|--|--|--|--|--|--|--|
/// |byte1| Success - Maximum QoS 0|0 |0 |0 |0 |0 |0 |0 |0 |
/// |byte2| Success - Maximum QoS 1|0 |0 |0 |0 |0 |0 |0 |1 |
/// |byte3| Success - Maximum QoS 2|0 |0 |0 |0 |0 |1 |0 |0 |
/// |byte4|      Failure           |1 |0 |0 |0 |0 |0 |0 |0 |
///
/// 完整SUBACK报文如下：
///
/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 1   | 0   | 0   | 1   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 1   | 0   | 1   |
/// | byte3 | 报  | 文  | 标   | 识  | 符   | M   | S   | B   |
/// | byte4 | 报  | 文  | 标   | 识  | 符   | L   | S   | B   |
/// | byte5 | x   | 0   | 0   | 0   | 0   |  0   | x   | x   |
///
#[derive(Debug, Clone)]
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
                let qos = fixed_header.qos();
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = GeneralVariableHeader::decode(&mut bytes, qos);
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

    use crate::common::coder::{Decoder, Encoder};
    use crate::v4::builder::MqttMessageBuilder;

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
        let _ = resp.encode(&mut bytes);
        let resp = SubAck::decode(bytes.into());
        match resp {
            Ok(sub) => println!("新的sub = {:?}", sub),
            Err(e) => println!("解码异常 {}", e),
        }
    }
}
