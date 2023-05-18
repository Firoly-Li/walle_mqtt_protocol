use std::slice::Iter;

use super::{
    publish::{FOUR_BYTE_MAX_LEN, ONE_BYTE_MAX_LEN, THREE_BYTE_MAX_LEN, TWO_BYTE_MAX_LEN},
    Decoder, Encoder,
};
use crate::{
    error::ProtoError,
    v4::decoder::{check_fixed_header_options, check_fixed_header_type, check_remain_length},
    MessageType, QoS,
};

use crate::error::BuildError;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/**
 固定报头
```text
         7                          3                          0
         +--------------------------+--------------------------+
byte 1   | MQTT Control Packet Type | Flags for each type      |
         +--------------------------+--------------------------+
         |         Remaining Bytes Len  (1/2/3/4 bytes)        |
         +-----------------------------------------------------+
```
*/
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct FixedHeader {
    // 消息类型
    message_type: MessageType,
    // dup
    dup: Option<bool>,
    // 消息质量
    qos: Option<QoS>,
    // 遗嘱清除
    retain: Option<bool>,
    // 剩余长度
    remaining_length: usize,
    // fixed_header本身的长度
    len: usize,
}

impl FixedHeader {
    pub fn new(
        message_type: MessageType,
        dup: Option<bool>,
        qos: Option<QoS>,
        retain: Option<bool>,
        remaining_length: usize,
        len: usize,
    ) -> Self {
        Self {
            message_type,
            dup,
            qos,
            retain,
            remaining_length,
            len,
        }
    }
    // message_type
    pub fn message_type(&self) -> MessageType {
        self.message_type.clone()
    }
    // dup
    pub fn dup(&self) -> Option<bool> {
        self.dup
    }
    // qos
    pub fn qos(&self) -> Option<QoS> {
        self.qos
    }
    // retain
    pub fn retain(&self) -> Option<bool> {
        self.retain
    }
    // remaining_length
    pub fn remaining_length(&self) -> usize {
        self.remaining_length
    }
    pub fn set_remaining_length(&mut self, remaining_length: usize) {
        self.remaining_length = remaining_length;
    }
    // 返回fixed_header的长度
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    // 根据mqtt报文首字节校验fixed_header是否正确,check方法执行之后byte的首字节去掉了
    pub fn check(mut byte1: &mut Bytes) -> Result<MessageType, BuildError> {
        let b = byte1.get_u8();
        FixedHeader::check_with_u8(b)
    }

    // 根据mqtt报文首字节校验fixed_header是否正确,check方法执行之后byte的首字节去掉了
    pub fn check_with_u8(byte1: u8) -> Result<MessageType, BuildError> {
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
            n => Err(BuildError::MessageTypeError(n as usize)),
        }
    }
}

//////////////////////////////////////////////////////
/// 为FixedHeader实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for FixedHeader {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        match self.message_type {
            MessageType::CONNECT => connect_fixed_header_encode(self, buffer),
            MessageType::CONNACK => connack_fixed_header_encode(self, buffer),
            MessageType::PUBLISH => publish_fixed_header_encode(self, buffer),
            MessageType::PUBACK => puback_fixed_header_encode(self, buffer),
            MessageType::PUBREC => pubrec_fixed_header_encode(self, buffer),
            MessageType::PUBREL => pubrel_fixed_header_encode(self, buffer),
            MessageType::PUBCOMP => pubcomp_fixed_header_encode(self, buffer),
            MessageType::SUBSCRIBE => subscribe_fixed_header_encode(self, buffer),
            MessageType::SUBACK => suback_fixed_header_encode(self, buffer),
            MessageType::UNSUBSCRIBE => unsubscribe_fixed_header_encode(self, buffer),
            MessageType::UNSUBACK => unsuback_fixed_header_encode(self, buffer),
            MessageType::DISCONNECT => disconnect_fixed_header_encode(self, buffer),
            MessageType::PINGREQ => pingreq_fixed_header_encode(self, buffer),
            MessageType::PINGRESP => pingresp_fixed_header_encode(self, buffer),
        }
    }
}

/// 对connect报文中固定头的编码
fn connect_fixed_header_encode(
    fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    buffer.put_u8(0b0001_0000);
    if fixed_header.remaining_length() > 268_435_455 {
        return Err(ProtoError::OutOfMaxRemainingLength(
            fixed_header.remaining_length,
        ));
    }
    let mut done = false;
    let mut x = fixed_header.remaining_length();
    let mut count = 0;
    while !done {
        let mut byte = (x % 128) as u8;
        x /= 128;
        if x > 0 {
            byte |= 128;
        }
        buffer.put_u8(byte);
        count += 1;
        done = x == 0;
    }
    Ok(count)
}
/// 对connack报文中固定头的编码
fn connack_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b0010_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0010);
    Ok(2)
}
/// 对pingreq报文中固定头的编码
fn pingreq_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b1100_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0000);
    Ok(2)
}
/// 对pingresq报文中固定头的编码
fn pingresp_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    buffer.put_u8(0b1101_0000);
    buffer.put_u8(0b0000_0000);
    Ok(2)
}

/// 对publish报文中固定头的编码
fn publish_fixed_header_encode(
    fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    let mut resp: usize = 0;
    // 写入byte1
    let mut byte1: u8 = 0b0000_0000;
    let qos = fixed_header.qos().unwrap();
    match qos {
        QoS::AtMostOnce => {}
        QoS::AtLeastOnce => byte1 = 0b0011_0000 | 0b0000_0010,
        QoS::ExactlyOnce => byte1 = 0b0011_0000 | 0b0000_0100,
    }
    let dup = fixed_header.dup().unwrap();
    if dup == true {
        byte1 = byte1 | 0b0000_1000;
    }
    let retain = fixed_header.retain().unwrap();
    if retain == true {
        byte1 = byte1 | 0b0000_0001;
    }
    buffer.put_u8(byte1);
    resp += 1;
    // 写入剩余长度
    let remaining_length = fixed_header.remaining_length();
    let encode_resp = encode_remaining_len(remaining_length, buffer);
    match encode_resp {
        Ok(size) => Ok(resp + size),
        Err(e) => Err(e),
    }
}
/// 对puback报文中固定头的编码
fn puback_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b0100_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0010);
    Ok(2)
}
/// 对pubrec报文中固定头的编码
fn pubrec_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b0101_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0010);
    Ok(2)
}
/// 对pubrel报文中固定头的编码
fn pubrel_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b0110_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0010);
    Ok(2)
}
/// 对pubcomp报文中固定头的编码
fn pubcomp_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b0111_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0010);
    Ok(2)
}
/// 对subscribe报文中固定头的编码
fn subscribe_fixed_header_encode(
    fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    let mut resp: usize = 0;
    // 写入byte1
    let mut byte1: u8 = 0b1000_0010;
    buffer.put_u8(byte1);
    resp += 1;
    // 写入剩余长度
    let remaining_length = fixed_header.remaining_length();
    let encode_resp = encode_remaining_len(remaining_length, buffer);
    match encode_resp {
        Ok(size) => Ok(resp + size),
        Err(e) => Err(e),
    }
}
/// 对suback报文中固定头的编码
fn suback_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b1001_0000);
    buffer.put_u8(0b0000_0011);
    Ok(2)
}
/// 对unsubscribe报文中固定头的编码,
fn unsubscribe_fixed_header_encode(
    fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    buffer.put_u8(0b1010_0010);
    let remaining_length = fixed_header.remaining_length();
    let encode_resp = encode_remaining_len(remaining_length, buffer);
    match encode_resp {
        Ok(size) => Ok(1 + size),
        Err(e) => Err(e),
    }
}
/// 对unsuback报文中固定头的编码
fn unsuback_fixed_header_encode(
    fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b1011_0000);
    let remaining_length = fixed_header.remaining_length();
    let encode_resp = encode_remaining_len(remaining_length, buffer);
    match encode_resp {
        Ok(size) => Ok(1 + size),
        Err(e) => Err(e),
    }
}
/// 对disconnect报文中固定头的编码
fn disconnect_fixed_header_encode(
    _fixed_header: &FixedHeader,
    buffer: &mut BytesMut,
) -> Result<usize, ProtoError> {
    // fixed_header 的第一个字节
    buffer.put_u8(0b1110_0000);
    // connAck报文的剩余长度是2个字节
    buffer.put_u8(0b0000_0000);
    Ok(2)
}

//////////////////////////////////////////////////////
/// 固定报头构造器
//////////////////////////////////////////////////////
pub struct FixedHeaderBuilder {
    // 消息类型
    message_type: MessageType,
    // dup
    dup: Option<bool>,
    // 消息质量
    qos: Option<QoS>,
    // 遗嘱清除
    retain: Option<bool>,
    // 剩余长度
    remaining_length: usize,
}

impl FixedHeaderBuilder {
    pub fn new() -> Self {
        Self {
            message_type: MessageType::CONNECT,
            dup: Some(false),
            qos: None,
            retain: Some(false),
            remaining_length: 0,
        }
    }
    pub fn from_message_type(message_type: MessageType) -> Self {
        Self {
            message_type,
            dup: None,
            qos: None,
            retain: None,
            remaining_length: 0,
        }
    }

    // 构建connect报文
    pub fn connect(mut self) -> FixedHeaderBuilder {
        self.message_type = MessageType::CONNECT;
        self
    }
    // 构建conn_ack报文
    pub fn conn_ack(mut self) -> Self {
        self.message_type = MessageType::CONNACK;
        self.remaining_length = 2;
        self
    }

    // 构建dis_connect报文
    pub fn dis_connect(mut self) -> Self {
        self.message_type = MessageType::DISCONNECT;
        self
    }

    // 构建ping_req报文
    pub fn ping_req(mut self) -> Self {
        self.message_type = MessageType::PINGREQ;
        self
    }

    // 构建ping_resp报文
    pub fn ping_resp(mut self) -> Self {
        self.message_type = MessageType::PINGRESP;
        self
    }

    // 构建publish报文
    pub fn publish(mut self) -> Self {
        self.message_type = MessageType::PUBLISH;
        self
    }

    // 构建pub_ack报文
    pub fn pub_ack(mut self) -> Self {
        self.message_type = MessageType::PUBACK;
        self
    }

    // 构建pub_rec报文
    pub fn pub_rec(mut self) -> Self {
        self.message_type = MessageType::PUBREC;
        self
    }

    // 构建pub_rel报文
    pub fn pub_rel(mut self) -> Self {
        self.message_type = MessageType::PUBREL;
        self
    }

    // 构建pub_comp报文
    pub fn pub_comp(mut self) -> Self {
        self.message_type = MessageType::PUBCOMP;
        self
    }

    // 构建subscribe报文
    pub fn subscribe(mut self) -> Self {
        self.message_type = MessageType::SUBSCRIBE;
        self.qos = Some(QoS::AtLeastOnce);
        self
    }
    // 构建sub_ack报文
    pub fn sub_ack(mut self) -> Self {
        self.message_type = MessageType::SUBACK;
        self
    }

    // 构建un_subscribe报文
    pub fn un_subscribe(mut self) -> Self {
        self.message_type = MessageType::UNSUBSCRIBE;
        self
    }

    // 构建un_suback报文
    pub fn un_suback(mut self) -> Self {
        self.message_type = MessageType::UNSUBACK;
        self
    }
}

//////////////////////////////////////////////////////
/// 数据类型构造器
//////////////////////////////////////////////////////
impl FixedHeaderBuilder {
    // 设置dup
    pub fn dup(mut self, dup: Option<bool>) -> Self {
        self.dup = dup;
        self
    }
    // 设置qos
    pub fn qos(mut self, qos: Option<QoS>) -> Self {
        self.qos = qos;
        self
    }
    // 设置retain
    pub fn retain(mut self, retain: Option<bool>) -> Self {
        self.retain = retain;
        self
    }
    // 设置remaining_length
    pub fn remaining_length(mut self, remaining_length: usize) -> Self {
        self.remaining_length = remaining_length;
        self
    }

    pub fn build(self) -> Result<FixedHeader, ProtoError> {
        let resp = remaining_length_len(self.remaining_length);
        match resp {
            Ok(size) => Ok(FixedHeader {
                message_type: self.message_type,
                dup: self.dup,
                qos: self.qos,
                retain: self.retain,
                remaining_length: self.remaining_length,
                len: size + 1,
            }),
            Err(e) => Err(e),
        }
    }
}

// 通过剩余长度计算出剩余长度的值所占的字节数
fn remaining_length_len(remaining_length: usize) -> Result<usize, ProtoError> {
    if remaining_length < ONE_BYTE_MAX_LEN {
        Ok(1)
    } else if remaining_length < TWO_BYTE_MAX_LEN {
        Ok(2)
    } else if remaining_length < THREE_BYTE_MAX_LEN {
        Ok(3)
    } else if remaining_length < FOUR_BYTE_MAX_LEN {
        Ok(4)
    } else {
        Err(ProtoError::NotKnow)
    }
}

//TODO 添加注释, 这里可能有问题
fn encode_remaining_len(remaining_len: usize, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
    let mut resp: usize = 0;
    // 1、判断remaining_len的范围
    if remaining_len < ONE_BYTE_MAX_LEN {
        buffer.put_u8(remaining_len as u8);
        resp = 1;
    } else if remaining_len < TWO_BYTE_MAX_LEN {
        let byte2_data = remaining_len / 128;
        let byte1 = remaining_len % 128;
        let byte1 = byte1 + 128;
        let byte2 = byte2_data % 128;
        buffer.put_u8(byte1 as u8);
        buffer.put_u8(byte2 as u8);
        resp = 2;
    } else if remaining_len < THREE_BYTE_MAX_LEN {
        let byte2_data = remaining_len / 128;
        let byte3_data = byte2_data / 128;
        let byte1 = remaining_len % 128;
        let byte1 = byte1 + 128;
        let byte2 = byte2_data % 128;
        let byte2 = byte2 + 128;
        let byte3 = byte3_data % 128;
        buffer.put_u8(byte1 as u8);
        buffer.put_u8(byte2 as u8);
        buffer.put_u8(byte3 as u8);
        resp = 3;
    } else if remaining_len < FOUR_BYTE_MAX_LEN {
        let byte2_data = remaining_len / 128;
        let byte3_data = byte2_data / 128;
        let byte4_data = byte3_data / 128;
        let byte1 = remaining_len % 128;
        let byte1 = byte1 + 128;
        let byte2 = byte2_data % 128;
        let byte2 = byte2 + 128;
        let byte3 = byte3_data % 128;
        let byte3 = byte3 + 128;
        let byte4 = byte4_data % 128;
        buffer.put_u8(byte1 as u8);
        buffer.put_u8(byte2 as u8);
        buffer.put_u8(byte3 as u8);
        buffer.put_u8(byte4 as u8);
        resp = 4;
    } else {
        return Err(ProtoError::OutOfMaxRemainingLength(remaining_len));
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::FixedHeaderBuilder;
    use tracing::info;

    #[test]
    fn builder_should_work() {
        let fixed_header = FixedHeaderBuilder::new()
            .connect()
            .dup(Some(true))
            .qos(Some(crate::QoS::AtLeastOnce))
            .retain(Some(false))
            .remaining_length(12)
            .build();
        info!("fixed_header = {:?}", fixed_header);
    }
}
