use bytes::{Bytes, BytesMut};

use super::decoder::read_fixed_header;
use super::fixed_header::FixedHeader;
use super::{fixed_header::FixedHeaderBuilder, Decoder, Encoder};
use crate::error::ProtoError;
use crate::MessageType;

/// 心跳应答报文
///
/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 1   | 1   | 0   | 1   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0   |

#[derive(Default, Debug, Clone)]
pub struct PingResp {
    fixed_header: FixedHeader,
}

impl PingResp {
    pub fn new() -> Self {
        let fixed_header = FixedHeaderBuilder::new()
            .ping_resp()
            .dup(Some(false))
            .qos(Some(crate::QoS::AtMostOnce))
            .retain(Some(false))
            .remaining_length(0)
            .build();
        Self {
            fixed_header: fixed_header.unwrap(),
        }
    }
    pub fn from_fixed_header(fixed_header: FixedHeader) -> Self {
        Self { fixed_header }
    }
}

//////////////////////////////////////////////////////
/// 为PingResp实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for PingResp {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        self.fixed_header.encode(buffer)
    }
}

//////////////////////////////////////////////////////
/// 为PingResp实现Decoder trait
//////////////////////////////////////////////////////
impl Decoder for PingResp {
    type Item = PingResp;
    type Error = ProtoError;
    fn decode(mut stream: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = read_fixed_header(&mut stream);
        match resp {
            Ok(fixed_header) => {
                if fixed_header.message_type() == MessageType::PINGRESP {
                    Ok(PingResp::from_fixed_header(fixed_header))
                } else {
                    Err(ProtoError::NotKnow)
                }
            }
            Err(err) => Err(err),
        }
    }
}
