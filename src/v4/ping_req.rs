use bytes::Bytes;
use bytes::BytesMut;

use super::decoder::read_fixed_header;
use super::Decoder;

use super::fixed_header::FixedHeader;
use super::fixed_header::FixedHeaderBuilder;
use super::Encoder;
use crate::error::ProtoError;
use crate::MessageType;
/////////////////////////////////////////////////////////////
/// 心跳报文
/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 1   | 1   | 0   | 0   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0   |
/////////////////////////////////////////////////////////////
#[derive(Default, Debug, Clone)]
pub struct PingReq {
    // 固定报头
    fixed_header: FixedHeader,
}

impl PingReq {
    pub fn new() -> Self {
        let fixed_header = FixedHeaderBuilder::new()
            .ping_req()
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
/// 为PingReq实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for PingReq {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        self.fixed_header.encode(buffer)
    }
}
//////////////////////////////////////////////////////
/// 为PingReq实现Decoder trait
//////////////////////////////////////////////////////
impl Decoder for PingReq {
    type Item = PingReq;
    type Error = ProtoError;
    fn decode(mut stream: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = read_fixed_header(&mut stream);
        match resp {
            Ok(fixed_header) => {
                if fixed_header.message_type() == MessageType::PINGREQ {
                    Ok(PingReq::from_fixed_header(fixed_header))
                } else {
                    Err(ProtoError::NotKnow)
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, BytesMut};

    use crate::v4::Encoder;

    use super::PingReq;

    #[test]
    fn test() {
        let mut buffer = BytesMut::new();
        let ping_req = PingReq::new();
        ping_req.encode(&mut buffer);
        // let buf = buffer.freeze();
        println!("buffer = {:#?}", &buffer[..]);
    }
}
