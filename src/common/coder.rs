use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::ProtoError;

/// 编码
pub trait Encoder: Sync + Send + 'static {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError>;
}

/// 解码
pub trait Decoder: Sync + Send + 'static {
    // 定义的返回类型
    type Item;
    // 错误类型
    type Error;
    // 将bytes解析为对应的报文
    fn decode(bytes: Bytes) -> Result<Self::Item, Self::Error>;
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
