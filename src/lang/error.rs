use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} Is not a valid key in this position")]
    InvalidKeyType(super::types::Ty),
    #[error("Form terminated at length {0} but there are {1} unterminated paired tokens")]
    FormEarlyEnd(usize, usize),
    #[error("Unbalanced String")]
    Unbalanced,
}
