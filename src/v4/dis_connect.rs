use super::{decoder, fixed_header, Decoder, Encoder};
use crate::error::ProtoError;
use crate::v4::fixed_header::FixedHeader;
use bytes::{Bytes, BytesMut};

/// 断开连接报文
/// | Bit   | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | ----- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | byte1 | 1   | 1   | 1   | 0   | 0   | 0   | 0   | 0   |
/// | byte2 | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0   |
#[derive(Default, Debug, Clone)]
pub struct DisConnect {
    fixed_header: FixedHeader,
}
impl DisConnect {
    pub fn new(fixed_header: FixedHeader) -> Self {
        Self { fixed_header }
    }
}

impl Encoder for DisConnect {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        self.fixed_header.encode(buffer)
    }
}

impl Decoder for DisConnect {
    type Item = DisConnect;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => Ok(DisConnect::new(fixed_header)),
            Err(e) => Err(e),
        }
    }
}
