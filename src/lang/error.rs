use {super::types::Ty, thiserror::Error};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} Is not a valid key in this position")]
    InvalidKeyType(Ty),
    #[error("Form terminated at length {0} but there are {1} unterminated paired tokens")]
    FormEarlyEnd(usize, usize),
    #[error("Unbalanced String")]
    Unbalanced,
    #[error("unknown eval error: {0}")]
    UnknownEval(Ty),
    #[error("Symbol '{0}' has not been defined")]
    UnknownSymbol(String),
    #[error("Incorrect type: {0}")]
    InvalidType(Ty),
    #[error("Unknown Typecheck Error")]
    UnknownTypeCheck,
}
