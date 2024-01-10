use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("parse string to integer fail")]
    ParseIntError,
    /// parse integer to opcode fail
    #[error("decode binary opcode to asm fail")]
    ParseOpcodeError,
    /// interpreter not use single value for return
    #[error("interpreter not use single value for return")]
    InterpreterReturnSingle,

    #[error("U32 range check fail, value out range")]
    U32RangeCheckFail,

    #[error("assert fail: reg: {0}, value: {1}")]
    AssertFail(u64, u64),

    #[error("Memory visit invalid, bound addr: {0}")]
    MemVistInv(u64),

    #[error("Tape visit invalid, bound addr: {0}")]
    TapeVistInv(u64),

    #[error("pc visit invalid, over bound addr: {0}")]
    PcVistInv(u64),

    #[error("Tload flag is invalid: {0}")]
    TloadFlagInvalid(u64),

    #[error("Pubkey is invalid: {0}")]
    PubKeyInvalid(String),

    #[error("Signature is invalid: {0}")]
    SignatureInvalid(String),

    #[error("Message is invalid: {0}")]
    MessageInvalid(String),
}
