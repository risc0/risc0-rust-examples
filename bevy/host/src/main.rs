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

use std::io::prelude::*;

use bevy_core::Outputs;
use methods::{RUN_TURN_ELF, RUN_TURN_ID};
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::Prover;

fn main() {
    // Make the prover.
    let mut prover = Prover::new(RUN_TURN_ELF, RUN_TURN_ID)
        .expect("Prover should be constructed from matching method code & ID");

    // Run prover & generate receipt
    let receipt = prover.run().expect("Code should be provable");

    let journal = &receipt.journal;
    println!("Journal: {:?}", journal);
    //let outputs: Outputs = from_slice(&journal).expect("Journal should contain an Outputs object");
    //println!("\nGame state with hash\n  provably moved primary entity by {} units on the x axis\n", outputs.position);
}
