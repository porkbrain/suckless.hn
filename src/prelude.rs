pub use crate::models::*;
pub use tokio::prelude::*;

use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
