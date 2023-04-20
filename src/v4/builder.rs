use std::ops::Sub;

use super::{
    conn_ack::{ConnAck, ConnAckType},
    connect::{Connect, ConnectFlags, ConnectVariableHeader, LastWill, Login},
    dis_connect::DisConnect,
    fixed_header::{self, FixedHeaderBuilder},
    publish::{Publish, PublishVariableHeader},
    sub_ack::SubAck,
    subscribe::Subscribe,
    un_subscribe::UnSubscribe,
    GeneralVariableHeader,
};
use crate::v4::pub_ack::PubAck;
use crate::v4::pub_comp::PubComp;
use crate::v4::pub_rec::PubRec;
use crate::v4::pub_rel::PubRel;
use crate::{error::ProtoError, MqttVersion, QoS, Topic, PROTOCOL_NAME};
use bytes::Bytes;
use crate::v4::un_suback::UnSubAck;

/// mqtt 报文构造器
pub struct MqttMessageBuilder {}

impl MqttMessageBuilder {
    pub fn connect() -> ConnectBuilder {
        ConnectBuilder::new()
    }
    pub fn disconnect() -> DisconnectBuilder {
        DisconnectBuilder::new()
    }
    pub fn publish() -> PublishBuilder {
        PublishBuilder::new()
    }
    pub fn pub_rel() -> PubRelBuilder {
        PubRelBuilder::new()
    }
    pub fn pub_rec() -> PubRecBuilder {
        PubRecBuilder::new()
    }
    pub fn pub_comp() -> PubCompBuilder {
        PubCompBuilder::new()
    }
    pub fn pub_ack() -> PubAckBuilder {
        PubAckBuilder::new()
    }
    pub fn conn_ack() -> ConnAckBuilder {
        ConnAckBuilder::new()
    }
    pub fn subscribe() -> SubscribeBuilder {
        SubscribeBuilder::new()
    }
    pub fn sub_ack() -> SubAckBuilder {
        SubAckBuilder::new()
    }
    pub fn unsubscriber() -> UnsubscriberBuilder {
        UnsubscriberBuilder::new()
    }
    pub fn unsub_ack() -> UnsubAckBuilder { UnsubAckBuilder::new() }
}

///////////////////////////////////
/// Connect Builder
///////////////////////////////////
pub struct ConnectBuilder {
    protocol_level: MqttVersion,
    keep_alive: u16,
    client_id: String,
    clean_session: bool,
    username: Option<String>,
    password: Option<String>,
    will_qos: QoS,
    will_topic: Option<String>,
    retain: bool,
    will_message: Option<Bytes>,
}

impl ConnectBuilder {
    pub fn new() -> Self {
        Self {
            protocol_level: MqttVersion::V4,
            keep_alive: 60,
            client_id: String::new(),
            clean_session: false,
            username: None,
            password: None,
            will_qos: QoS::AtMostOnce,
            will_topic: None,
            retain: false,
            will_message: None,
        }
    }
    pub fn protocol_level(mut self, protocol_level: MqttVersion) -> Self {
        self.protocol_level = protocol_level;
        self
    }
    pub fn keep_alive(mut self, keep_alive: u16) -> Self {
        self.keep_alive = keep_alive;
        self
    }
    pub fn client_id(mut self, client_id: &str) -> Self {
        self.client_id = client_id.to_string();
        self
    }
    pub fn clean_session(mut self, clean_session: bool) -> Self {
        self.clean_session = clean_session;
        self
    }
    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }
    pub fn will_qos(mut self, will_qos: QoS) -> Self {
        self.will_qos = will_qos;
        self
    }
    pub fn will_topic(mut self, will_topic: &str) -> Self {
        self.will_topic = Some(will_topic.to_string());
        self
    }
    pub fn retain(mut self, retain: bool) -> Self {
        self.retain = retain;
        self
    }
    pub fn will_message(mut self, will_message: Bytes) -> Self {
        self.will_message = Some(will_message);
        self
    }
    pub fn build(self) -> Result<Connect, ProtoError> {
        // 初始化值
        let client_id = self.client_id;
        let username_flag = false;
        let password_flag = false;
        let mut will_flag = false;
        let will_retain = false;
        let will_qos = QoS::AtMostOnce;
        let clean_session = false;
        let will_topic = self.will_topic.clone();
        if self.will_topic.is_some() && self.will_message.is_some() {
            will_flag = true;
        }
        // 构建ConnFlags
        let conn_flags = ConnectFlags::new(
            username_flag,
            password_flag,
            will_retain,
            will_qos,
            will_flag,
            clean_session,
        );
        // 构建可变报头
        let variable_header = ConnectVariableHeader::new(
            PROTOCOL_NAME.to_string(),
            self.protocol_level,
            conn_flags,
            self.keep_alive,
        );
        let mut login = None;
        // 构建 Login
        if self.username.is_some() && self.password.is_some() {
           login = Some(Login::new(self.username.unwrap(), self.password.unwrap()));
        }
        // 计算login_len
        let login_len = match &login {
            Some(login) => login.len(),
            None => 0
        };
        // 构建LastWill
        let last_will: Option<LastWill> = match will_topic {
            Some(topic) =>{
                Some(LastWill::new(
            topic,
            self.will_message.unwrap(),
            self.will_qos,
            self.retain,))
            },
            None => None
        };
        // 计算last_will_len
        let last_will_len = match &last_will {
            Some(t) => t.len(),
            None => 0
        };
        let remaining_length = {
            let mut len = 2 + PROTOCOL_NAME.len() // protocol name
                + 1  // protocol version
                + 1  // connect flags
                + 2; // keep alive
            len += 2 + client_id.len();
            // last will len
            len += last_will_len;
            // username and password len
            len += login_len;
            len
        };
        let fixed_header = FixedHeaderBuilder::new()
            .connect()
            .dup(Some(false))
            .qos(Some(QoS::AtMostOnce))
            .retain(Some(false))
            .remaining_length(remaining_length)
            .build();
        match fixed_header {
            Ok(fixed_header) => Ok(Connect {
                fixed_header,
                variable_header,
                client_id,
                last_will,
                login,
            }),
            Err(e) => Err(e),
        }
    }
}

///////////////////////////////////
/// ConnAck Builder
///////////////////////////////////
pub struct ConnAckBuilder {
    conn_ack_type: ConnAckType,
}

impl ConnAckBuilder {
    fn new() -> Self {
        Self {
            conn_ack_type: ConnAckType::Success,
        }
    }

    pub fn conn_ack_type(mut self, conn_ack_type: ConnAckType) -> Self {
        self.conn_ack_type = conn_ack_type;
        self
    }

    pub fn build(&self) -> ConnAck {
        ConnAck::new(self.conn_ack_type.clone()).unwrap()
    }
}

///////////////////////////////////
/// Publish Builder
///////////////////////////////////
pub struct PublishBuilder {
    topic: String,
    message_id: usize,
    qos: Option<QoS>,
    retain: Option<bool>,
    dup: Option<bool>,
    payload: Bytes,
}

impl PublishBuilder {
    fn new() -> Self {
        Self {
            topic: String::new(),
            message_id: 0,
            qos: None,
            retain: None,
            dup: None,
            payload: Bytes::new(),
        }
    }

    pub fn topic(mut self, topic: &str) -> Self {
        self.topic = topic.to_string();
        self
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn qos(mut self, qos: QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    pub fn retain(mut self, retain: bool) -> Self {
        self.retain = Some(retain);
        self
    }

    pub fn dup(mut self, dup: bool) -> Self {
        self.dup = Some(dup);
        self
    }
    pub fn payload_string(mut self, payload: String) -> Self {
        self.payload = Bytes::from(payload);
        self
    }
    pub fn payload_str(mut self, payload: &str) -> Self {
        self.payload = Bytes::from(payload.to_string());
        self
    }
    pub fn payload(mut self, payload: Bytes) -> Self {
        self.payload = payload;
        self
    }


    pub fn build(self) -> Result<Publish, ProtoError> {
        //1、构建fixed_header
        let fixed_header = FixedHeaderBuilder::new()
            .publish()
            .dup(self.dup)
            .retain(self.retain)
            .qos(self.qos)
            .build();
        //2、构建variable_header
        let variable_header = PublishVariableHeader::new(self.topic, Some(self.message_id));
        //3、计算剩余长度
        let remaining_length = variable_header.variable_header_len() + self.payload.len() + 2;
        //4、构建Publish
        match fixed_header {
            Ok(mut fixed_header) => {
                fixed_header.set_remaining_length(remaining_length);
                Ok(Publish::new(fixed_header, variable_header, self.payload))
            }
            Err(e) => Err(e),
        }
    }
}

///////////////////////////////////
/// PubAck Builder
///////////////////////////////////
pub struct PubAckBuilder {
    message_id: usize,
}

impl PubAckBuilder {
    pub fn new() -> Self {
        Self { message_id: 0 }
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn build(&self) -> Result<PubAck, ProtoError> {
        Ok(PubAck::new(self.message_id))
    }
}

///////////////////////////////////
/// Disconnect Builder
///////////////////////////////////
pub struct DisconnectBuilder {}

impl DisconnectBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(&self) -> Result<DisConnect, ProtoError> {
        let resp = FixedHeaderBuilder::new().dis_connect().build();
        match resp {
            Ok(fixed_header) => Ok(DisConnect::new(fixed_header)),
            Err(e) => Err(e),
        }
    }
}

///////////////////////////////////
/// PubRel Builder
///////////////////////////////////
pub struct PubRelBuilder {
    message_id: usize,
}

impl PubRelBuilder {
    pub fn new() -> Self {
        Self { message_id: 0 }
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn build(&self) -> Result<PubRel, ProtoError> {
        Ok(PubRel::new(self.message_id))
    }
}

///////////////////////////////////
/// PubRec Builder
///////////////////////////////////
pub struct PubRecBuilder {
    message_id: usize,
}

impl PubRecBuilder {
    pub fn new() -> Self {
        Self { message_id: 0 }
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn build(&self) -> Result<PubRec, ProtoError> {
        Ok(PubRec::new(self.message_id))
    }
}

///////////////////////////////////
/// PubComp Builder
///////////////////////////////////
pub struct PubCompBuilder {
    message_id: usize,
}

impl PubCompBuilder {
    pub fn new() -> Self {
        Self { message_id: 0 }
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn build(&self) -> Result<PubComp, ProtoError> {
        Ok(PubComp::new(self.message_id))
    }
}

///////////////////////////////////
/// Subscriber Builder
///////////////////////////////////
pub struct SubscribeBuilder {
    topics: Vec<Topic>,
    message_id: usize,
}

impl SubscribeBuilder {
    pub fn new() -> Self {
        Self {
            topics: Vec::new(),
            message_id: 0,
        }
    }

    pub fn topics(mut self, topices: Vec<Topic>) -> Self {
        self.topics = topices;
        self
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn topic(mut self, topic: Topic) -> Self {
        self.topics.push(topic);
        self
    }

    pub fn build(mut self) -> Result<Subscribe, ProtoError> {
        if let (Ok(fixed_header), variable_header) = (
            FixedHeaderBuilder::new().subscribe().build(),
            GeneralVariableHeader::new(self.message_id),
        ) {
            return Ok(Subscribe::new(fixed_header, variable_header, self.topics));
        }
        Err(ProtoError::NotKnow)
    }
}

///////////////////////////////////
/// SubAck Builder
///////////////////////////////////
pub struct SubAckBuilder {
    qos: QoS,
    message_id: usize,
    pub acks: Vec<u8>,
}

impl SubAckBuilder {
    pub fn new() -> SubAckBuilder {
        SubAckBuilder {
            qos: QoS::AtMostOnce,
            message_id: 0,
            acks: Vec::new(),
        }
    }
    // pub fn qos(mut self, qos: QoS) -> Self {
    //     self.qos = qos;
    //     self
    // }
    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }
    pub fn acks(mut self, acks: Vec<u8>) -> Self {
        self.acks = acks;
        self
    }
    pub fn build(self) -> Result<SubAck, ProtoError> {
        let fixed_header = FixedHeaderBuilder::new().sub_ack().build();
        match fixed_header {
            Ok(mut fixed_header) => {
                fixed_header.set_remaining_length(2 + self.acks.len());
                let variable_header = GeneralVariableHeader::new(self.message_id);
                Ok(SubAck::new(fixed_header, variable_header, self.acks))
            }
            Err(e) => Err(e),
        }
    }
}

///////////////////////////////////
/// Unsubscriber Builder
///////////////////////////////////
pub struct UnsubscriberBuilder {
    message_id: usize,
    topices: Vec<String>,
}

impl UnsubscriberBuilder {
    pub fn new() -> Self {
        Self {
            message_id: 0,
            topices: Vec::new(),
        }
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn topices(mut self, topices: Vec<String>) -> Self {
        self.topices = topices;
        self
    }

    pub fn remaining_length(&self) -> usize {
        let iter = self.topices.iter();
        let mut len = 0;
        for temp in iter {
            let topic_len = temp.len() + 2;
            len += topic_len
        }
        len
    }

    pub fn build(&self) -> Result<UnSubscribe, ProtoError> {
        let resp = FixedHeaderBuilder::new().un_subscribe().build();
        match resp {
            Ok(mut fixed_header) => {
                let remaining_len = self.remaining_length();

                let variable_header = GeneralVariableHeader::new(self.message_id);
                fixed_header.set_remaining_length(remaining_len + variable_header.len());
                Ok(UnSubscribe::new(
                    fixed_header,
                    variable_header,
                    self.topices.clone(),
                ))
            }
            Err(e) => Err(e),
        }
    }
}

///////////////////////////////////
/// UnsubAck Builder
///////////////////////////////////
pub struct UnsubAckBuilder {
    message_id: usize,
}

impl UnsubAckBuilder {
    pub fn new() -> Self {
        Self {
            message_id: 0
        }
    }

    pub fn message_id(mut self, message_id: usize) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn build(mut self) -> Result<UnSubAck,ProtoError> {
        let resp = FixedHeaderBuilder::new().un_suback().build();
        match resp {
            Ok(mut fixed_header) => {
                let variable_header = GeneralVariableHeader::new(self.message_id);
                fixed_header.set_remaining_length(variable_header.len());
                Ok(UnSubAck::new(
                    fixed_header,
                    variable_header
                ))
            }
            Err(e) => Err(e),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::MqttMessageBuilder;
    use crate::v4::Encoder;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn build_connect_test() {
        let connect = MqttMessageBuilder::connect()
            .client_id("client_01")
            .keep_alive(10)
            .clean_session(true)
            .username("rump")
            .password("mq")
            .protocol_level(crate::MqttVersion::V4)
            .retain(false)
            .will_qos(crate::QoS::AtLeastOnce)
            .will_topic("/a")
            .will_message(Bytes::from_static(b"offline"))
            .build();
        // println!("connect = {:?}", connect);
        let mut bytes = BytesMut::new();
        connect.unwrap().encode(&mut bytes).unwrap();
        // println!("bytes = {:?}", bytes);
    }

    #[test]
    fn test() {
        let b = Bytes::from_static(b"this is will message!").len();
        println!("b = {}", b);
    }
}
