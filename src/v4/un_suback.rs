use super::{
    fixed_header::{FixedHeader, FixedHeaderBuilder},
    Decoder, Encoder,
};
use crate::{error::ProtoError, MessageType};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::v4::builder::MqttMessageBuilder;
use crate::v4::{decoder, GeneralVariableHeader, VariableDecoder};

#[derive(Debug)]
pub struct UnSubAck {
    fixed_header: FixedHeader,
    variable_header: GeneralVariableHeader,
}

impl UnSubAck {
    pub fn new(fixed_header: FixedHeader,
               variable_header: GeneralVariableHeader) -> Self {
        Self { fixed_header, variable_header }
    }
    pub fn message_id(&self) -> usize {
        self.variable_header.message_id
    }
}

//////////////////////////////////////////////////////
/// 为UnSubAck实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for UnSubAck {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        if let Ok(_resp) = self.fixed_header.encode(buffer) {
            buffer.put_u16(self.variable_header.message_id as u16);
            return Ok(4);
        }
        Err(ProtoError::NotKnow)
    }
}

//////////////////////////////////////////////////////
/// 为PubComp实现Decoder trait
//////////////////////////////////////////////////////
impl Decoder for UnSubAck {
    type Item = UnSubAck;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                if let Ok(variable_header) = GeneralVariableHeader::decode(&mut bytes) {
                    return Ok(UnSubAck {
                        fixed_header,
                        variable_header,
                    });
                }
                Err(ProtoError::DecodeGeneralVariableHeaderError)
            },
            Err(e) => Err(e)
        }
    }
}
//     // 1、判断bytes的长度，PubComp报文只有固定的4个字节
//     if bytes.len() != 4 {
//         return Err(ProtoError::NotKnow);
//     }
//     let resp = decoder::read_fixed_header(&mut bytes);
//     
//     
//     if let Ok(message_type) = FixedHeader::check(&mut bytes) {
//         if message_type == MessageType::UNSUBACK {
//             let _b1 = bytes.get_u16();
//             let message_id = bytes.get_u16();
//             Ok(MqttMessageBuilder::unsub_ack().message_id(message_id as usize).build())
//         } else {
//             Err(ProtoError::NotKnow)
//         }
//     } else {
//         Err(ProtoError::NotKnow)
//     }
// }

