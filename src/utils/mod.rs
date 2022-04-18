use validator::Validate;

use crate::error::Result;

pub mod encryption;
pub mod envelope;
pub mod jwt;

pub fn _validate_payload<T: Validate>(payload: &T) -> Result<()> {
    Ok(payload.validate()?)
}
