
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error(" Out of gas")]
    OutOfGas
}


impl RunError {}