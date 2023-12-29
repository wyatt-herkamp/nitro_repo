use sha2::{digest::FixedOutput, Digest, Sha512};

use crate::utils::base64_utils;

pub fn hash_token(value: impl AsRef<str>) -> String {
    let mut wrapper = Sha512::default();
    wrapper.update(value.as_ref().as_bytes());
    let array = wrapper.finalize_fixed();
    base64_utils::encode(array.as_slice())
}
