use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{error::ProtoError, MqttVersion, QoS, PROTOCOL_NAME};

use super::{
    decoder::{self, *},
    fixed_header::FixedHeader,
    Decoder, Encoder, VariableDecoder,
};
//////////////////////////////////////////////////////
/// Connect报文
//////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub struct Connect {
    // 固定报头
    pub fixed_header: FixedHeader,
    // 可变报头
    pub variable_header: ConnectVariableHeader,
    // 客户端id
    pub client_id: String,
    // 客户端遗嘱信息
    pub last_will: Option<LastWill>,
    // 连接信息
    pub login: Option<Login>,
}

impl Connect {
    pub fn new(
        fixed_header: FixedHeader,
        variable_header: ConnectVariableHeader,
        client_id: String,
        last_will: Option<LastWill>,
        login: Option<Login>,
    ) -> Self {
        Self {
            fixed_header,
            variable_header,
            client_id,
            last_will,
            login,
        }
    }

    pub fn len(&self) -> usize {
        let mut len = 2 + PROTOCOL_NAME.len() // protocol name
                              + 1            // protocol version
                              + 1            // connect flags
                              + 2; // keep alive

        len += 2 + self.client_id.len();
        // last will len
        if let Some(last_will) = &self.last_will {
            len += last_will.len();
        }
        // username and password len
        if let Some(login) = &self.login {
            len += login.len();
        }
        len
    }
}

//////////////////////////////////////////////////////
/// 为Connect实现Encoder trait
//////////////////////////////////////////////////////
impl Encoder for Connect {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, ProtoError> {
        let count = self.fixed_header.encode(buffer).unwrap();
        // variable_header
        write_mqtt_string(buffer, PROTOCOL_NAME);

        // 写protocol_level
        match self.variable_header.protocol_level {
            MqttVersion::V4 => buffer.put_u8(0x04),
            MqttVersion::V5 => buffer.put_u8(0x05),
        }
        // connect_flags
        let mut connect_flags = 0;
        if self.variable_header.connect_flags.clean_session {
            connect_flags |= 0x02;
        }
        match &self.login {
            Some(_login) => {
                connect_flags |= 0xc0;
            }
            None => {}
        }
        if self.variable_header.connect_flags.will_retain {
            connect_flags |= 0x20;
        }
        match self.variable_header.connect_flags.will_qos {
            QoS::AtMostOnce => {}
            QoS::AtLeastOnce => {
                connect_flags |= 0x08;
            }
            QoS::ExactlyOnce => {
                connect_flags |= 0x10;
            }
        }
        match &self.last_will {
            Some(_last_will) => {
                connect_flags |= 0x04;
            }
            None => {}
        }
        buffer.put_u8(connect_flags);
        buffer.put_u16(self.variable_header.keep_alive());
        write_mqtt_string(buffer, &self.client_id);
        if let Some(last_will) = &self.last_will {
            connect_flags |= last_will.write(buffer)?;
        }
        if let Some(login) = &self.login {
            connect_flags |= login.write(buffer);
        }
        Ok(self.len())
    }
}

//////////////////////////////////////////////////////
/// 为Connect实现Decoder trait
//////////////////////////////////////////////////////
impl Decoder for Connect {
    type Item = Connect;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        // 读取fixed_header
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = ConnectVariableHeader::decode(&mut bytes);
                match resp {
                    Ok(variable_header) => {
                        // connect报文的variable_header是固定的8个字节
                        let client_id = read_mqtt_string(&mut bytes)?;
                        // bytes.advance(variable_header.len());
                        let last_will =
                            LastWill::read_last_will(&mut bytes, &variable_header.connect_flags);
                        let login = Login::read_login(&mut bytes, &variable_header.connect_flags);
                        let connect = Connect::new(
                            fixed_header,
                            variable_header,
                            client_id,
                            last_will,
                            login,
                        );
                        Ok(connect)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(ProtoError::NotKnow),
        }
    }
}

//////////////////////////////////////////////
/// ConnectVariableHeader
/////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectVariableHeader {
    // 协议名称
    protocol_name: String,
    // 协议级别
    protocol_level: MqttVersion,
    // 连接标志
    connect_flags: ConnectFlags,
    // 心跳
    keep_alive: u16,
}

impl ConnectVariableHeader {
    pub fn new(
        protocol_name: String,
        protocol_level: MqttVersion,
        connect_flags: ConnectFlags,
        keep_alive: u16,
    ) -> Self {
        Self {
            protocol_name,
            protocol_level,
            connect_flags,
            keep_alive,
        }
    }
    pub fn protocol_name(&self) -> &str {
        &self.protocol_name
    }
    pub fn protocol_level(&self) -> MqttVersion {
        self.protocol_level.clone()
    }
    pub fn connect_flags(&self) -> &ConnectFlags {
        &self.connect_flags
    }
    pub fn keep_alive(&self) -> u16 {
        self.keep_alive
    }
    pub fn len(&self) -> usize {
        8
    }
}

impl VariableDecoder for ConnectVariableHeader {
    type Item = ConnectVariableHeader;
    // 构建variable_header
    fn decode(stream: &mut Bytes) -> Result<ConnectVariableHeader, ProtoError> {
        let resp = read_mqtt_string(stream);
        match resp {
            Ok(protocol_name) => {
                if protocol_name != PROTOCOL_NAME {
                    Err(ProtoError::NotKnow)
                } else {
                    let protocol_level = read_u8(stream).unwrap();
                    let protocol = match protocol_level {
                        4 => MqttVersion::V4,
                        5 => MqttVersion::V5,
                        _num => return Err(ProtoError::NotKnow),
                    };
                    let connect_flags_u8 = read_u8(stream)?;
                    let connect_flags = ConnectFlags::from_u8(connect_flags_u8);
                    let keep_alive = read_u16(stream)?;
                    match connect_flags {
                        Ok(flags) => Ok(ConnectVariableHeader::new(
                            PROTOCOL_NAME.to_owned(),
                            protocol,
                            flags,
                            keep_alive,
                        )),
                        Err(e) => Err(e),
                    }
                }
            }
            Err(e) => Err(ProtoError::NotKnow),
        }
    }
}

/**
连接标志位，连接标志字节包含了一些用于指定MQTT链接行为的参数，它还指出了有效载荷中的字段是否存在

| bit  |       7        |       6       |       5     |     4    |     3    |     2     |       1       |     0    |
| ---- | -------------- | ------------- | ----------- | -------- | -------- | --------- | ------------- | -------- |
|     | User Name Flag  | Password Flag | Will Retain | Will Qos | Will QoS | Will Flag | Clean Session | Reserved |
|byte8|         x       |       x       |       x     |    x      |    x    |      x    |        x      |    0     |

 */
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectFlags {
    username_flag: bool,
    password_flag: bool,
    will_retain: bool,
    will_qos: QoS,
    will_flag: bool,
    clean_session: bool,
}

impl ConnectFlags {
    pub fn new(
        username_flag: bool,
        password_flag: bool,
        will_retain: bool,
        will_qos: QoS,
        will_flag: bool,
        clean_session: bool,
    ) -> Self {
        Self {
            username_flag,
            password_flag,
            will_retain,
            will_qos,
            will_flag,
            clean_session,
        }
    }

    pub fn clean_session(&self) -> bool {
        self.clean_session
    }
    pub fn will_qos(&self) -> QoS {
        self.will_qos
    }
    pub fn will_flag(&self) -> bool {
        self.will_flag
    }

    fn from_u8(byte: u8) -> Result<Self, ProtoError> {
        // username_flag
        let username_flag = byte >> 7 != 0;
        // password_flag
        let password_flag = (byte & 0b0100_0000) != 0;
        // will_retain
        let will_retain = (byte & 0b0010_0000) != 0;
        let will_qos_value = (byte & 0b0001_1000) >> 3;
        let will_qos = match will_qos_value {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => return Err(ProtoError::QoSError(will_qos_value)),
        };
        // will_flag
        let will_flag = (byte & 0b0000_0100) != 0;
        // clean_session
        let clean_session = (byte & 0b10) != 0;
        Ok(Self {
            username_flag,
            password_flag,
            will_retain,
            will_qos,
            will_flag,
            clean_session,
        })
    }
}

/// 客户端登陆信息
#[derive(Debug, Clone, PartialEq)]
pub struct Login {
    // 账号信息
    pub username: String,
    // 密码信息
    pub password: String,
}

impl Login {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
    pub fn len(&self) -> usize {
        let mut len = 0;
        if !self.username.is_empty() {
            len += 2 + self.username.len();
        }
        if !self.password.is_empty() {
            len += 2 + self.password.len();
        }
        len
    }
    pub fn write(&self, buffer: &mut BytesMut) -> u8 {
        let mut connect_flags = 0;
        if !self.username.is_empty() {
            connect_flags |= 0x80;
            write_mqtt_string(buffer, &self.username);
        }

        if !self.password.is_empty() {
            connect_flags |= 0x40;
            write_mqtt_string(buffer, &self.password);
        }
        connect_flags
    }
}
impl Login {
    fn read_login(stream: &mut Bytes, connect_flags: &ConnectFlags) -> Option<Self> {
        let mut username = String::new();
        let mut password = String::new();
        if connect_flags.username_flag {
            username = read_mqtt_string(stream).unwrap();
        }
        if connect_flags.password_flag {
            password = read_mqtt_string(stream).unwrap();
        }
        if username.is_empty() && password.is_empty() {
            return None;
        }
        Some(Login::new(username, password))
    }
}

/// 客户端遗嘱信息
#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    // 主题
    pub topic_name: String,
    // 遗嘱消息的内容
    pub message: Bytes,
    // 遗嘱消息的质量
    pub qos: QoS,
    // 遗嘱保留
    pub retain: bool,
}

impl LastWill {
    pub fn new(topic_name: String, message: Bytes, qos: QoS, retain: bool) -> Self {
        Self {
            topic_name,
            message,
            qos,
            retain,
        }
    }
    pub fn len(&self) -> usize {
        let mut len = 0;
        len += 2 + self.topic_name.len() + 2 + self.message.len();
        len
    }

    pub fn write(&self, buffer: &mut BytesMut) -> Result<u8, ProtoError> {
        let mut connect_flags = 0;
        connect_flags |= 0x04 | (self.qos as u8) << 3;
        if self.retain {
            connect_flags |= 0x20;
        }
        write_mqtt_string(buffer, &self.topic_name);
        write_mqtt_bytes(buffer, &self.message);
        Ok(connect_flags)
    }
}

impl LastWill {
    // 读取last_will的内容，这里的stream就是connect报文中的payload内容，fixed_header和variable_header已经去除
    fn read_last_will(stream: &mut Bytes, connect_flags: &ConnectFlags) -> Option<Self> {
        match connect_flags.will_flag {
            true => {
                let will_topic = read_mqtt_string(stream).unwrap();
                let will_payload = read_mqtt_bytes(stream).unwrap();
                let last_will = LastWill::new(
                    will_topic,
                    will_payload,
                    connect_flags.will_qos,
                    connect_flags.will_retain,
                );
                Some(last_will)
            }
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use crate::{
        v4::{
            builder::MqttMessageBuilder,
            fixed_header::{FixedHeader, FixedHeaderBuilder},
            Decoder, Encoder,
        },
        PROTOCOL_NAME,
    };

    use super::{Connect, ConnectFlags, ConnectVariableHeader, LastWill};

    #[test]
    fn test() {
        let fixed_header = FixedHeaderBuilder::new().connect().build();
    }

    fn build_fixed_header() -> Option<FixedHeader> {
        let fixed_header = FixedHeaderBuilder::new()
            .connect()
            .dup(Some(true))
            .qos(Some(crate::QoS::AtLeastOnce))
            .retain(Some(false))
            .remaining_length(12)
            .build();
        match fixed_header {
            Ok(fixed_header) => Some(fixed_header),
            Err(_) => None,
        }
    }

    fn build_connect_variable_header() -> Option<ConnectVariableHeader> {
        let conn_flags = ConnectFlags::new(true, true, false, crate::QoS::AtMostOnce, true, false);
        let variable_header = ConnectVariableHeader::new(
            PROTOCOL_NAME.to_owned(),
            crate::MqttVersion::V4,
            conn_flags,
            60,
        );
        Some(variable_header)
    }
    // 创建一个connect报文
    fn build_connect() -> Option<Connect> {
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
        Some(connect.unwrap())
    }

    #[test]
    fn encode_and_decode_for_connect_should_be_work() {
        let connect = build_connect().unwrap();
        println!("原始connect = {:?}", connect);
        let mut bytes = BytesMut::new();
        let mut bytes1 = BytesMut::new();
        connect.encode(&mut bytes).unwrap();
        println!("bytes = {:?}", bytes);
        let connect1 = Connect::decode(bytes.freeze());
        match connect1 {
            Ok(connect) => {
                println!("connect1 = {:?}", connect);
                connect.encode(&mut bytes1).unwrap();
                println!(
                    " encode_and_decode_for_connect_should_be_work bytes1  = {:?}",
                    bytes1
                );
                let connect2 = Connect::decode(bytes1.into()).unwrap();
                println!("connect2 = {:?}", connect2);
            }
            Err(err) => println!("编解码出错"),
        }
    }
}
