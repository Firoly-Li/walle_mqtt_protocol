use bytes::{Buf, BufMut, Bytes, BytesMut};
use super::{
    fixed_header::{FixedHeader, FixedHeaderBuilder},
    Decoder, Encoder,
};
use crate::error::ProtoError;
use crate::v4::{decoder, GeneralVariableHeader, VariableDecoder};

/// 发布确认报文
/// PUBACK报文分为两部分，固定头和可变头，其中固定头的内容是固定的，
/// 可变头中存放的是消息id(message_id)
///
/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 0   | 1   | 0   | 0   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 0   | 1   | 0   |
/// | byte3 | 报   | 文   | 标  | 识   | 符  | M   | S   | B   |
/// | byte4 | 报   | 文   | 标  | 识   | 符  | L   | S   | B   |
///
#[derive(Debug)]
pub struct PubAck {
    fixed_header: FixedHeader,
    variable_header: GeneralVariableHeader,
}

impl PubAck {
    pub fn new(message_id: usize) -> Self {
        Self {
            fixed_header: FixedHeaderBuilder::new().pub_rel().build().unwrap(),
            variable_header: GeneralVariableHeader::new(message_id),
        }
    }

    pub fn message_id(&self) -> usize {
        self.variable_header.message_id
    }
}

//////////////////////////////////////////////////////
/// 为PubAck实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for PubAck {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let fixed_header = FixedHeaderBuilder::new().pub_ack().build();
        match fixed_header {
            Ok(fixed_header) => {
                if let Ok(_resp) = fixed_header.encode(buffer) {
                    buffer.put_u16(self.variable_header.message_id() as u16);
                    return Ok(4);
                }
                Err(ProtoError::EncodeVariableHeaderError)
            }
            Err(err) => Err(err),
        }
    }
}

//////////////////////////////////////////////////////
/// 为PubAck实现Decoder trait
//////////////////////////////////////////////////////
impl Decoder for PubAck {
    type Item = PubAck;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let qos = fixed_header.qos();
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = GeneralVariableHeader::decode(&mut bytes, qos);
                match resp {
                    Ok(variable_header) => Ok(PubAck {
                        fixed_header,
                        variable_header,
                    }),
                    Err(e) => return Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}
