/// Error during serialization and deserialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ProtoError {
    #[error("not know")]
    NotKnow,
    #[error("使用了错误的QoS值：{0}")]
    QoSError(u8),
    #[error("错误的fixed_header长度：{0}")]
    FixedHeaderLengthError(usize),
    #[error("错误的dup值：{0}")]
    DupValueError(u8),
    #[error("错误的retain值：{0}")]
    RetainValueError(u8),

    #[error("超出MQTT协议规定的最大长度：{0}")]
    OutOfMaxRemainingLength(usize),
    #[error("MQTT报文判断错误：{0}")]
    MessageTypeError(#[from] BuildError),
    #[error("读取topic出错！")]
    ReadTopicError,
    #[error("解码GeneralVariableHeader出错！")]
    DecodeGeneralVariableHeaderError,
    #[error("解码fixedHeader出错！")]
    DecodeFixedleHeaderError,
    #[error("编码variable_header错误！")]
    EncodeVariableheaderError,
    #[error("编码remaining_length错误！")]
    EncodeRemainingLengthError
}


/// 消息构建错误相关
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BuildError {
    #[error("超出MQTT协议规定的最大长度：{0}")]
    OutOfMaxRemainingLength(usize),
    #[error("MQTT报文判断错误：{0}")]
    MessageTypeError(usize),
}
