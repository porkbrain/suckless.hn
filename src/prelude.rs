pub use crate::models::*;

use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
