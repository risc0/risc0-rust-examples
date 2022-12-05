use risc0_zkp::core::sha::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Outputs {
    pub data: u32,
    pub hash: Digest,
}
