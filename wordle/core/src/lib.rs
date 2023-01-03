use risc0_zkp::core::sha::Digest;
use serde::{Deserialize, Serialize};

pub const WORD_LENGTH: usize = 5;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum LetterFeedback {
    LetterCorrect,
    LetterPresent,
    #[default]
    LetterMiss,
}

pub type WordFeedback = [LetterFeedback; WORD_LENGTH];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameState {
    pub correct_word_hash: Digest,
    pub feedback: WordFeedback,
}
