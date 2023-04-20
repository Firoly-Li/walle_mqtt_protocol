use super::fixed_header::{FixedHeader, FixedHeaderBuilder};
use crate::{error::ProtoError, MessageType, QoS};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::slice::Iter;

/// 从Bytes中读取固定报头
pub fn read_fixed_header(stream: &mut Bytes) -> Result<FixedHeader, ProtoError> {
    // 由于fixed_header的长度在2-5个字节之间，所以stream_len的长度必须要大与等于2
    let stream_len = stream.len();
    // println!("stream_len: {}", stream_len);
    if stream_len < 2 && stream_len > 5 {
        return Err(ProtoError::FixedHeaderLengthError(stream_len));
    }
    let mut iter = stream.iter();
    // 拿到首字节byte1
    let byte1 = iter.next().unwrap();
    // 确定fixed_header的类型
    let resp = check_fixed_header_type(byte1);
    match resp {
        Ok(message_type) => {
            // println!("message_type = {:?}", message_type);
            // 优先得到fixed_header（此时的fixed_header还没有计算剩余长度）
            let resp = check_fixed_header_options(byte1, message_type);
            // println!("response = {:?}", resp);
            match resp {
                Ok(fixed_header) => {
                    check_remain_length(iter, fixed_header)
                }
                Err(err) => Err(err),
            }
        }
        Err(e) => Err(e),
    }
}

pub fn parse_fixed_header(mut stream: Iter<u8>) -> Result<FixedHeader, ProtoError> {
    let stream_len = stream.len();
    if stream_len < 2 {
        return Err(ProtoError::NotKnow);
    }
    // 拿到首字节byte1
    let byte1 = stream.next().unwrap();
    // 确定fixed_header的类型
    let resp = check_fixed_header_type(byte1);
    match resp {
        Ok(message_type) => {
            // println!("message_type = {:?}", message_type);
            // 优先得到fixed_header（此时的fixed_header还没有计算剩余长度）
            let resp = check_fixed_header_options(byte1, message_type);
            // println!("response = {:?}", resp);
            match resp {
                Ok(fixed_header) => {
                    // println!("fixed_header = {:?}", fixed_header);
                    // 计算fixed_header的remaing_length)(剩余长度)
                    check_remain_length(stream, fixed_header)
                }
                Err(err) => Err(err),
            }
        }
        Err(e) => Err(e),
    }
}

/// 根据首字节校验fixed_header的类型
pub fn check_fixed_header_type(byte1: &u8) -> Result<MessageType, ProtoError> {
    // println!("check_fixed_header_type ----- byte1: {:08b}", byte1);
    match byte1 >> 4 {
        1 => Ok(MessageType::CONNECT),
        2 => Ok(MessageType::CONNACK),
        3 => Ok(MessageType::PUBLISH),
        4 => Ok(MessageType::PUBACK),
        5 => Ok(MessageType::PUBREC),
        6 => Ok(MessageType::PUBREL),
        7 => Ok(MessageType::PUBCOMP),
        8 => Ok(MessageType::SUBSCRIBE),
        9 => Ok(MessageType::SUBACK),
        10 => Ok(MessageType::UNSUBSCRIBE),
        11 => Ok(MessageType::UNSUBACK),
        12 => Ok(MessageType::PINGREQ),
        13 => Ok(MessageType::PINGRESP),
        14 => Ok(MessageType::DISCONNECT),
        _ => Err(ProtoError::NotKnow),
    }
}
/// 获取fixed_header的其他值：dup、qos、retain，不包括剩余长度
pub fn check_fixed_header_options(
    byte1: &u8,
    message_type: MessageType,
) -> Result<FixedHeader, ProtoError> {
    let mut dup: Option<bool> = Some(false);
    let mut qos: Option<QoS> = None;
    let mut retain: Option<bool> = Some(false);
    // 根据message_type创建制定的fixed_header_budiler
    let fixed_header_builder = FixedHeaderBuilder::from_message_type(message_type.clone());
    // 获取低4位数
    // println!("check_fixed_header_options -- byte1 = {}", byte1);
    let low_4 = byte1 & 0b0000_1111;
    // println!("check_fixed_header_options -- low_4 = {}", low_4);
    match message_type {
        MessageType::PUBLISH => {
            //处理b3位数据，这里决定了dup标识
            match low_4 >> 3 {
                0 => dup = Some(false),
                1 => dup = Some(true),
                x => return Err(ProtoError::DupValueError(x)),
            }

            //处理b2和b1位数据，这两位一般一起确定了QoS,和0b0000_0110进行与操作之后还要向右移1位
            match (low_4 & 0b0000_0110) >> 1 {
                0 => qos = Some(QoS::AtMostOnce),
                1 => qos = Some(QoS::AtLeastOnce),
                2 => qos = Some(QoS::ExactlyOnce),
                x => return Err(ProtoError::QoSError(x)),
            }
            //处理b0位数据，这里决定了retain标志
            match low_4 & 0b0000_0001 {
                0 => retain = Some(false),
                1 => retain = Some(true),
                x => return Err(ProtoError::RetainValueError(x)),
            }
            fixed_header_builder
                .dup(dup)
                .qos(qos)
                .retain(retain)
                .build()
        }
        MessageType::PUBREL | MessageType::SUBSCRIBE | MessageType::UNSUBSCRIBE => {
            //处理b3位数据，这里决定了dup标识
            match low_4 >> 3 {
                0 => dup = Some(false),
                1 => dup = Some(true),
                x => return Err(ProtoError::DupValueError(x)),
            };
            //处理b2和b1位数据，这两位一般一起确定了QoS
            match (low_4 & 0b0000_0110) >> 1 {
                1 => qos = None,
                x => return Err(ProtoError::NotKnow),
            };
            //处理b0位数据，这里决定了retain标志
            match low_4 & 0b0000_0001 {
                0 => retain = Some(false),
                1 => retain = Some(true),
                x => return Err(ProtoError::RetainValueError(x)),
            };
            fixed_header_builder
                .dup(dup)
                .qos(qos)
                .retain(retain)
                .build()
        }
        _ => match low_4 & 0b0000_1111 {
            0 => fixed_header_builder
                .dup(dup)
                .qos(qos)
                .retain(retain)
                .build(),
            _ => Err(ProtoError::NotKnow),
        },
    }
}
// 配置fixed_header的剩余长度，此时的stream已经去掉了byte1
pub fn check_remain_length(
    stream: Iter<u8>,
    mut fixed_header: FixedHeader,
) -> Result<FixedHeader, ProtoError> {
    let mut shift = 0;
    let mut len = 0;
    let mut fixed_header_len = 1;
    let mut done = false;
    for b in stream {
        fixed_header_len += 1;
        let byte = *b as usize;
        len += (byte & 0x7F) << shift;
        // stop when continue bit is 0
        done = (byte & 0x80) == 0;
        if done {
            break;
        }
        shift += 7;
        if shift > 21 {
            return Err(ProtoError::NotKnow);
        }
    }
    if !done {
        return Err(ProtoError::NotKnow);
    }
    fixed_header.set_remaining_length(len);
    fixed_header.set_len(fixed_header_len);
    Ok(fixed_header)
}

///读取数据到bytes
pub fn read_mqtt_bytes(stream: &mut Bytes) -> Result<Bytes, ProtoError> {
    let len = read_u16(stream)? as usize;
    if len > stream.len() {
        return Err(ProtoError::NotKnow);
    }
    Ok(stream.split_to(len))
}
///读取数据到字符串
pub fn read_mqtt_string(stream: &mut Bytes) -> Result<String, ProtoError> {
    let s = read_mqtt_bytes(stream)?;
    match String::from_utf8(s.to_vec()) {
        Ok(v) => Ok(v),
        Err(_e) => Err(ProtoError::NotKnow),
    }
}

pub fn read_u16(stream: &mut Bytes) -> Result<u16, ProtoError> {
    if stream.len() < 2 {
        return Err(ProtoError::NotKnow);
    }
    Ok(stream.get_u16())
}

pub fn read_u8(stream: &mut Bytes) -> Result<u8, ProtoError> {
    if stream.is_empty() {
        return Err(ProtoError::NotKnow);
    }
    Ok(stream.get_u8())
}

pub fn write_mqtt_bytes(stream: &mut BytesMut, bytes: &[u8]) {
    stream.put_u16(bytes.len() as u16);
    stream.extend_from_slice(bytes);
}

/// Serializes a string to stream
pub fn write_mqtt_string(stream: &mut BytesMut, string: &str) {
    write_mqtt_bytes(stream, string.as_bytes());
}
