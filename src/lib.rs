//! ADIF Parser Library
//!
//! A library for parsing ADIF (Amateur Data Interchange Format) files.
//! Supports the ADI format as specified in ADIF 3.1.6.

mod error;
mod parser;
mod types;

pub use error::AdifError;
pub use parser::parse_adi;
pub use types::{AdifFile, AdifHeader, DataType, Field, Record};
