use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::common::coder::{Decoder, Encoder};
use crate::error::ProtoError;
use crate::v5::connect::Properties;
use crate::v5::ConnectReasonCode;

/**
 * 连接回执报文
 */
#[derive(Debug, Clone)]
pub struct ConnAck {
    pub session_present: bool,
    pub reason_code: ConnectReasonCode,
    pub properties: Properties,
}

impl Encoder for ConnAck {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let start_pos = buffer.len();

        // 会话存在标志
        buffer.put_u8(self.session_present as u8);

        // 原因码
        buffer.put_u8(self.reason_code as u8);

        // 属性
        self.properties.encode(buffer)?;

        Ok(buffer.len() - start_pos)
    }
}

impl Decoder for ConnAck {
    type Item = ConnAck;
    type Error = ProtoError;

    fn decode(mut bytes: Bytes) -> Result<Self, ProtoError> {
        let session_present = bytes.get_u8() != 0;
        let reason_code = ConnectReasonCode::try_from(bytes.get_u8())?;
        let properties = Properties::decode(bytes)?;

        Ok(ConnAck {
            session_present,
            reason_code,
            properties,
        })
    }
}
