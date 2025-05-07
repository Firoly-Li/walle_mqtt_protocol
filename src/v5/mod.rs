use crate::error::ProtoError;

pub mod connect;
pub mod conn_ack;


/**
 * 连接原因码
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ConnectReasonCode {
    Success = 0,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    UnsupportedProtocolVersion = 0x84,
    // 其他原因码...
}
impl TryFrom<u8> for ConnectReasonCode {
    type Error = ProtoError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConnectReasonCode::Success),
            0x80 => Ok(ConnectReasonCode::UnspecifiedError),
            0x81 => Ok(ConnectReasonCode::MalformedPacket),
            0x82 => Ok(ConnectReasonCode::ProtocolError),
            0x83 => Ok(ConnectReasonCode::ImplementationSpecificError),
            0x84 => Ok(ConnectReasonCode::UnsupportedProtocolVersion),
            // 补充其他标准原因码
            code @ 0x85..=0xFF => Err(ProtoError::UnknownReasonCode(code)),
            _ => Err(ProtoError::InvalidReasonCode(value)),
        }
    }
}