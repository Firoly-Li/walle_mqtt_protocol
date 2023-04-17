pub mod builder;
pub mod conn_ack;
pub mod connect;
pub mod decoder;
pub mod dis_connect;
pub mod fixed_header;
pub mod ping_req;
pub mod ping_resp;
pub mod pub_ack;
pub mod pub_comp;
pub mod pub_rec;
pub mod pub_rel;
pub mod publish;
pub mod sub_ack;
pub mod subscribe;
pub mod un_suback;
pub mod un_subscribe;

use self::conn_ack::ConnAck;
use self::connect::Connect;
use self::dis_connect::DisConnect;
use self::ping_req::PingReq;
use self::ping_resp::PingResp;
use self::pub_ack::PubAck;
use self::pub_comp::PubComp;
use self::pub_rec::PubRec;
use self::pub_rel::PubRel;
use self::publish::Publish;
use self::sub_ack::SubAck;
use self::subscribe::Subscribe;
use self::un_suback::UnSubAck;
use self::un_subscribe::UnSubscribe;
use crate::error::ProtoError;
use bytes::{Buf, BufMut, Bytes, BytesMut};

use anyhow::Result;

/// MQTT报文，包含了MQTT-v3.1.1版本中的所有MQTT报文
#[derive(Debug)]
pub enum Packet {
    // 连接报文
    Connect(Connect),
    // 连接回执报文
    ConnAck(ConnAck),
    // 发布报文
    Publish(Publish),
    // 发布回执报文
    PubAck(PubAck),
    PubRel(PubRel),

    PubRec(PubRec),

    PubComp(PubComp),
    // 心跳报文
    PingReq(PingReq),
    // 心跳回执报文
    PingResp(PingResp),
    // 订阅报文
    Subscribe(Subscribe),
    // 订阅回执报文
    SubAck(SubAck),
    // 取消订阅报文
    UnSubscribe(UnSubscribe),
    // 取消订阅回执报文
    UnSubAck(UnSubAck),
    // 断开链接报文
    DisConnect(DisConnect),
}

/// 编码
pub trait Encoder: Sync + Send + 'static {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError>;
}

/// 解码
pub trait Decoder: Sync + Send + 'static {
    // 定义的返回类型
    type Item;
    type Error;
    // 将bytes解析为对应的报文
    fn decode(bytes: Bytes) -> Result<Self::Item, Self::Error>;
}


pub trait VariableDecoder: Sync + Send + 'static {
    // 定义的返回类型
    type Item;
    // 将bytes解析为对应的报文
    fn decode(bytes: &mut Bytes) -> Result<Self::Item, ProtoError>;
}

//////////////////////////////////////////////////////
/// 通用可变头，只有message_id
//////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct GeneralVariableHeader {
    message_id: usize,
}

impl GeneralVariableHeader {
    pub fn new(message_id: usize) -> Self {
        Self { message_id }
    }

    pub fn message_id(&self) -> usize {
        self.message_id
    }

    pub fn len(&self) -> usize {
        2
    }
}

//////////////////////////////////////////////////////
/// 为SubscribeVariableHeader实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for GeneralVariableHeader {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let message_id = self.message_id as u16;
        buffer.put_u16(message_id);
        Ok(2)
    }
}

//////////////////////////////////////////////////////
/// 为SubscribeVariableHeader实现Decoder trait
//////////////////////////////////////////////////////
impl VariableDecoder for GeneralVariableHeader {
    type Item = GeneralVariableHeader;

    fn decode(bytes: &mut Bytes) -> Result<Self::Item, ProtoError> {
        let message_id = bytes.get_u16() as usize;
        Ok(GeneralVariableHeader { message_id })
    }
}
