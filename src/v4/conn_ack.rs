use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::ProtoError;

use super::{
    decoder,
    fixed_header::{FixedHeader, FixedHeaderBuilder},
    Decoder, Encoder, VariableDecoder,
};

/// 链接回执报文
/**
 | Bit  | 7  | 6 | 5  | 4 | 3 | 2 | 1 | 0 |
 | -----| ---| ---|---|---|---|---|---|---|
 | byte1 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |
 | byte2 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
 | byte3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |sp |
 | byte4 | 连 |接 |返 |回 | 码 | C | R | C |
*/
#[derive(Debug, PartialOrd, Clone, PartialEq)]
pub struct ConnAck {
    fixed_header: FixedHeader,
    variable_header: ConnAckVariableHeader,
}

impl ConnAck {
    pub fn new(conn_ack_type: ConnAckType) -> Result<ConnAck, ProtoError> {
        let fixed_header = FixedHeaderBuilder::new().conn_ack().build();
        match fixed_header {
            Ok(f_header) => Ok(Self {
                fixed_header: f_header,
                variable_header: ConnAckVariableHeader::new(conn_ack_type),
            }),
            Err(e) => Err(e),
        }
    }
}

#[derive(PartialOrd, Debug, Clone, PartialEq)]
pub enum ConnAckType {
    // 连接成功
    Success,
    // 版本错误
    ProtoVersionError,
    // 不符合规定的clientId
    IdentifierRejected,
    // 服务不可用
    ServiceUnavailable,
    // 账号或者密码错误
    BadUsernameOrPassword,
    // 未授权
    NotAuthentication,
}
//////////////////////////////////////////////////////////
/// 为ConnAck实现Encoder trait
/////////////////////////////////////////////////////////
impl Encoder for ConnAck {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let count = self.fixed_header.encode(buffer);
        match count {
            Ok(count) => {
                if let Ok(v) = self.variable_header.encode(buffer) {
                    return Ok(count + v);
                }
                Err(ProtoError::EncodeVariableheaderError)
            }
            Err(e) => Err(e),
        }
    }
}
//////////////////////////////////////////////////////////
/// 为ConnAck实现Decoder trait
/////////////////////////////////////////////////////////
impl Decoder for ConnAck {
    type Item = ConnAck;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = ConnAckVariableHeader::decode(&mut bytes);
                match resp {
                    Ok(variable_header) => Ok(ConnAck {
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

#[derive(Debug, PartialOrd, Clone, PartialEq)]
pub struct ConnAckVariableHeader {
    session_present: bool,
    conn_ack_type: ConnAckType,
}

impl ConnAckVariableHeader {
    pub fn new(conn_ack_type: ConnAckType) -> Self {
        Self {
            session_present: false,
            conn_ack_type,
        }
    }
}

//////////////////////////////////////////////////////////
/// 为ConnAckVariableHeader实现Encoder trait
/////////////////////////////////////////////////////////
impl Encoder for ConnAckVariableHeader {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        match &self.conn_ack_type {
            ConnAckType::Success => {
                buffer.put_u8(0b0000_0000);
                buffer.put_u8(0b0000_0000);
                Ok(2)
            }
            ConnAckType::ProtoVersionError => {
                buffer.put_u8(0b0000_0000);
                buffer.put_u8(0b0000_0001);
                Ok(2)
            }
            ConnAckType::IdentifierRejected => {
                buffer.put_u8(0b0000_0000);
                buffer.put_u8(0b0000_0010);
                Ok(2)
            }
            ConnAckType::ServiceUnavailable => {
                buffer.put_u8(0b0000_0000);
                buffer.put_u8(0b0000_0011);
                Ok(2)
            }
            ConnAckType::BadUsernameOrPassword => {
                buffer.put_u8(0b0000_0000);
                buffer.put_u8(0b0000_0100);
                Ok(2)
            }
            ConnAckType::NotAuthentication => {
                buffer.put_u8(0b0000_0000);
                buffer.put_u8(0b0000_0101);
                Ok(2)
            }
            // ConnAckType::Failed => {
            //     buffer.put_u8(0b0000_0000);
            //     buffer.put_u8(0b0000_0110);
            //     Ok(2)
            // }
        }
    }
}

//////////////////////////////////////////////////////////
/// 为 ConnAckVariableHeader 实现 VariableDecoder trait
/////////////////////////////////////////////////////////
impl VariableDecoder for ConnAckVariableHeader {
    type Item = ConnAckVariableHeader;

    fn decode(bytes: &mut Bytes) -> Result<Self::Item, ProtoError> {
        let b1 = bytes.get_u8();
        if b1 == 0 {
            let b2 = bytes.get_u8();
            let con_ack_type = match b2 {
                0b0000_0000 => ConnAckType::Success,
                0b0000_0001 => ConnAckType::ProtoVersionError,
                0b0000_0010 => ConnAckType::IdentifierRejected,
                0b0000_0011 => ConnAckType::ServiceUnavailable,
                0b0000_0100 => ConnAckType::BadUsernameOrPassword,
                0b0000_0101 => ConnAckType::NotAuthentication,
                _ => {
                    return Err(ProtoError::NotKnow);
                }
            };
            Ok(ConnAckVariableHeader::new(con_ack_type))
        } else {
            Err(ProtoError::NotKnow)
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::v4::{builder::MqttMessageBuilder, Decoder, Encoder};

    use super::ConnAck;

    #[test]
    fn encode_and_decode_for_connack_should_be_work() {
        let resp = MqttMessageBuilder::conn_ack()
            .conn_ack_type(super::ConnAckType::NotAuthentication)
            .build();
        println!("conn_ack: {:?}", resp);
        let mut buffer = BytesMut::new();
        let _count = resp.encode(&mut buffer);
        let conn_ack = ConnAck::decode(buffer.freeze()).unwrap();
        println!("conn_ack: {:?}", conn_ack);
    }
}
