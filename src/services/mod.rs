pub mod feature_flag_handlers;
pub mod environment_handlers;

use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct ServiceError {
    message: String,
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}