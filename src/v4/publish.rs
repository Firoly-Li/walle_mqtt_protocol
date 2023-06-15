use bytes::{Buf, BufMut, Bytes, BytesMut};
use tracing::{debug, info};

use crate::error::ProtoError;
use crate::QoS;

use super::{
    decoder::{self, read_mqtt_string, read_u16},
    fixed_header::FixedHeader,
    Decoder, Encoder, VariableDecoder,
};

/// 一个字节表示的最大长度
pub const ONE_BYTE_MAX_LEN: usize = 127;
/// 两个字节表示的最大长度
pub const TWO_BYTE_MAX_LEN: usize = 16383;
/// 三个字节表示的最大长度
pub const THREE_BYTE_MAX_LEN: usize = 2097151;
/// 四个字节表示的最大长度
pub const FOUR_BYTE_MAX_LEN: usize = 268435455;

/// 一个完整的publish报文的格式如下：
///
/// | 字节 | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   | 数值 | 备注     |
/// | ---- | --- | --- | --- | --- | --- | --- | --- | --- | ---- | -------- |
/// | 1    | 0   | 0   | 1   | 1   | 0   | 0   | 1   | 0   | 50   | 首字节   |
/// | 2    | 0   | 0   | 1   | 0   | 0   | 0   | 1   | 1   | 19   | 剩余长度 |
/// | 3    | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0   | 0    |          |
/// | 4    | 0   | 0   | 0   | 0   | 0   | 1   | 0   | 1   | 5    |          |
/// | 5    | 0   | 0   | 1   | 0   | 1   | 1   | 1   | 1   | 47   | /        |
/// | 6    | 0   | 1   | 1   | 1   | 0   | 1   | 0   | 0   | 116  | t        |
/// | 7    | 0   | 1   | 1   | 0   | 0   | 1   | 0   | 1   | 101  | e        |
/// | 8    | 0   | 1   | 1   | 1   | 0   | 0   | 1   | 1   | 115  | s        |
/// | 9    | 0   | 1   | 1   | 1   | 0   | 1   | 0   | 0   | 116  | t        |
/// | 10   | 0   | 1   | 1   | 1   | 1   | 0   | 1   | 0   | 122  |          |
/// | 11   | 0   | 1   | 1   | 1   | 0   | 0   | 0   | 0   | 112  |          |
/// | 12   | 0   | 0   | 1   | 1   | 0   | 0   | 0   | 1   | 49   | 1        |
/// | 13   | 0   | 0   | 1   | 1   | 0   | 0   | 1   | 0   | 50   | 2        |
/// | 14   | 0   | 0   | 1   | 1   | 0   | 0   | 1   | 1   | 51   | 3        |
/// | 15   | 0   | 0   | 1   | 1   | 0   | 1   | 0   | 0   | 52   | 4        |
/// | 16   | 0   | 0   | 1   | 1   | 0   | 1   | 0   | 1   | 53   | 5        |
/// | 17   | 0   | 0   | 1   | 1   | 0   | 1   | 1   | 0   | 54   | 6        |
/// | 18   | 0   | 0   | 1   | 1   | 0   | 1   | 1   | 1   | 55   | 7        |
/// | 19   | 0   | 0   | 1   | 1   | 1   | 0   | 0   | 0   | 56   | 8        |
/// | 20   | 0   | 0   | 1   | 1   | 1   | 0   | 0   | 1   | 57   | 9        |
/// | 21   | 0   | 0   | 1   | 1   | 0   | 0   | 0   | 0   | 48   | 0        |

#[derive(Debug, Clone)]
pub struct Publish {
    // 固定报头
    fixed_header: FixedHeader,
    // 可变报头
    variable_header: PublishVariableHeader,
    // payload 有效载荷
    payload: Bytes,
}

impl Publish {
    pub fn new(
        fixed_header: FixedHeader,
        variable_header: PublishVariableHeader,
        payload: Bytes,
    ) -> Self {
        Self {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn fixed_header(&self) -> FixedHeader {
        self.fixed_header.clone()
    }

    pub fn variable_header(&self) -> PublishVariableHeader {
        self.variable_header.clone()
    }

    pub fn payload(&self) -> Bytes {
        self.payload.clone()
    }

    /// 更新message_id,并且把QoS改为AtLeastOnce
    /// todo 其他两种QoS会出错
    pub fn update(self, message_id: usize) -> Self {
        let fixed_header = self.fixed_header.clone();
        // fixed_header.set_qos(QoS::AtLeastOnce);
        let variable_header = self.variable_header.clone().update_message_id(message_id);
        let payload = self.payload();
        Self {
            fixed_header,
            variable_header,
            payload,
        }
    }
}

//////////////////////////////////////////////////////////
/// 为Publish实现Encoder trait
/////////////////////////////////////////////////////////
impl Encoder for Publish {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let resp = self.fixed_header.encode(buffer);
        debug!("fixed_handler buffer = {:?}", buffer);
        let qos = self.fixed_header.qos().unwrap();
        match resp {
            Ok(fixed_header_len) => {
                let resp = self.variable_header.encode(buffer);
                match resp {
                    Ok(variable_header_len) => {
                        debug!("fixed_handler + variable_headler buffer = {:?}", buffer);
                        buffer.put(self.payload());
                        debug!("buffer = {:?}", buffer);
                        let resp = fixed_header_len + variable_header_len + self.payload().len();
                        Ok(resp)
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
}

//////////////////////////////////////////////////////////
/// 为Publish实现Decoder trait
/////////////////////////////////////////////////////////
impl Decoder for Publish {
    type Item = Publish;
    type Error = ProtoError;
    fn decode(mut bytes: Bytes) -> Result<Self::Item, ProtoError> {
        // 读取fixed_header
        let resp = decoder::read_fixed_header(&mut bytes);
        match resp {
            Ok(fixed_header) => {
                let qos = fixed_header.qos();
                let variable_header_index = fixed_header.len();
                bytes.advance(variable_header_index);
                // 读取variable_header
                let resp = PublishVariableHeader::decode(&mut bytes, qos);
                match resp {
                    Ok(variable_header) => Ok(Publish {
                        fixed_header,
                        variable_header,
                        payload: bytes,
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

//////////////////////////////////////////////
/// PublishVariableHeader
/////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct PublishVariableHeader {
    // variable_header的长度
    variable_header_len: usize,
    // topic
    topic: String,
    // message_id
    message_id: Option<usize>,
}
impl PublishVariableHeader {
    pub fn new(topic: String, message_id: Option<usize>, qos: Option<QoS>) -> Self {
        Self {
            variable_header_len: Self::variable_len(topic.as_str(), qos),
            topic,
            message_id,
        }
    }

    //
    fn variable_len(topic: &str, qos: Option<QoS>) -> usize {
        match qos {
            Some(qos) => {
                if qos == QoS::AtMostOnce {
                    topic.len() + 2
                } else {
                    topic.len() + 4
                }
            }
            None => topic.len() + 2,
        }
    }
    pub fn variable_header_len(&self) -> usize {
        self.variable_header_len
    }
    pub fn topic(&self) -> String {
        self.topic.clone()
    }
    pub fn message_id(&self) -> Option<usize> {
        self.message_id
    }
    pub fn update_message_id(mut self, message_id: usize) -> Self {
        self.message_id = Some(message_id);
        self
    }
}

//////////////////////////////////////////////////////////
/// 为PublishVariableHeader实现VariableDecode trait
/////////////////////////////////////////////////////////
impl VariableDecoder for PublishVariableHeader {
    type Item = PublishVariableHeader;

    fn decode(bytes: &mut Bytes, qos: Option<QoS>) -> Result<Self::Item, ProtoError> {
        // let topic_resp = read_mqtt_string(bytes);
        // let message_id_resp = read_u16(bytes);
        // if let (Ok(topic), Ok(message_id)) = (topic_resp, message_id_resp) {
        //     return Ok(PublishVariableHeader::new(topic, Some(message_id.into())));
        // }
        // Err(ProtoError::NotKnow)

        let topic_resp = read_mqtt_string(bytes);
        match topic_resp {
            Ok(topic) => match qos {
                Some(qos) => {
                    if qos == QoS::AtMostOnce {
                        return Ok(PublishVariableHeader::new(
                            topic,
                            None,
                            Some(QoS::AtMostOnce),
                        ));
                    } else {
                        let message_id = read_u16(bytes).unwrap();
                        return Ok(PublishVariableHeader::new(
                            topic,
                            Some(message_id.into()),
                            Some(qos),
                        ));
                    }
                }
                None => {
                    return Ok(PublishVariableHeader::new(topic, None, None));
                }
            },
            Err(e) => Err(e),
        }
    }
}

//////////////////////////////////////////////////////////
/// 为PublishVariableHeader实现Encoder trait
/////////////////////////////////////////////////////////
// todo 这里有个问题，PublishVariableHeader的长度是受到QoS影响的
impl Encoder for PublishVariableHeader {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        debug!("encode PublishVariableHandler");
        let topic_len = self.topic.len();
        debug!("topic_len = {}", topic_len);
        buffer.put_u16(topic_len as u16);
        let topic = self.topic.clone();
        debug!("topic = {:?}", topic.as_bytes());
        buffer.put(topic.as_bytes());
        let message_id = self.message_id;
        match message_id {
            Some(msg_id) => {
                buffer.put_u16(msg_id as u16);
                debug!("variable_header_len = {}", self.variable_header_len());
                Ok(self.variable_header_len())
            }
            None => Ok(self.variable_header_len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use crate::v4::{builder::MqttMessageBuilder, publish::Publish, Decoder, Encoder};

    #[test]
    fn publish_to_bytes() {
        if let Ok(publish) = MqttMessageBuilder::publish()
            .dup(false)
            .qos(crate::QoS::AtMostOnce)
            .retain(false)
            .topic("/test")
            .payload_str("hello world !")
            .build()
        {
            let remaining_len = publish.fixed_header.remaining_length();
            let qos = publish.fixed_header.qos();
            let topic = publish.variable_header.topic();

            // encode
            let mut buffer = BytesMut::new();
            publish.encode(&mut buffer);
            println!("buffer = {:?}", buffer);
            //decode
            if let Ok(new_publish) = Publish::decode(buffer.freeze()) {
                let new_remaining_len = new_publish.fixed_header.remaining_length();
                let new_qos = new_publish.fixed_header.qos();
                let new_topic = new_publish.variable_header.topic();
                assert_eq!(remaining_len, new_remaining_len);
                assert_eq!(qos, new_qos);
                assert_eq!(topic, new_topic);
            }
        }
    }

    #[test]
    fn publish_to_bytes_test() {
        if let Ok(publish) = MqttMessageBuilder::publish()
            .dup(false)
            .qos(crate::QoS::AtMostOnce)
            .message_id(1) // 当qos=0的时候设置message_id也是无效的
            .retain(false)
            .topic("/test")
            .payload_str("hello world !")
            .build()
        {
            let remaining_len = publish.fixed_header.remaining_length();
            let qos = publish.fixed_header.qos().unwrap();
            let topic = publish.variable_header.topic();
            let payload = publish.payload();

            // encode
            let mut buffer = BytesMut::new();
            publish.encode(&mut buffer);
            println!("buffer = {:?}", buffer);

            let publish1 = MqttMessageBuilder::publish()
                .dup(false)
                .retain(false)
                .message_id(1)
                .topic(topic.as_str())
                .qos(qos)
                .payload(payload)
                .build()
                .unwrap();
            let mut buffer1 = BytesMut::new();
            publish1.encode(&mut buffer1);
            assert_eq!(buffer, buffer1);
        }
    }
}
