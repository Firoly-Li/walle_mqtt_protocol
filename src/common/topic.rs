use bytes::{BufMut, Bytes, BytesMut};

use crate::{QoS, common::coder, error::ProtoError};

use super::coder::Encoder;

/////////////////////////////////////////////////////////////////////////
/// topic,客户端与服务端做信息交互的时候给消息做的标签
/////////////////////////////////////////////////////////////////////////
#[derive(Debug, Default, Clone, PartialOrd, Eq, PartialEq)]
pub struct Topic {
    name: String,
    qos: QoS,
    name_len: usize,
}
impl Topic {
    pub fn new(name: String, qos: QoS) -> Self {
        Self {
            name: name.clone(),
            qos,
            name_len: name.len(),
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn qos(&self) -> QoS {
        self.qos
    }
    pub fn name_len(&self) -> usize {
        self.name_len
    }
}

impl Topic {
    pub fn read_topics(stream: &mut Bytes) -> Result<Vec<Topic>, ProtoError> {
        let mut resp: Vec<Topic> = Vec::new();
        while !stream.is_empty() {
            if let (Ok(topic_name), Ok(qos)) =
                (coder::read_mqtt_string(stream), coder::read_u8(stream))
            {
                let qos = QoS::try_from(qos);
                match qos {
                    Ok(qos) => {
                        let topic = Topic::new(topic_name, qos);
                        resp.push(topic);
                    }
                    Err(e) => return Err(e),
                }
            } else {
                return Err(ProtoError::ReadTopicError);
            }
        }
        Ok(resp)
    }
}

impl Encoder for Topic {
    fn encode(&self, buffer: &mut BytesMut) -> Result<usize, ProtoError> {
        let topic_len = self.name_len;
        buffer.put_u16(topic_len as u16);
        buffer.put_slice(self.name.as_bytes());
        buffer.put_u8(self.qos as u8);
        Ok(topic_len + 3)
    }
}
