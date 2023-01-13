// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
