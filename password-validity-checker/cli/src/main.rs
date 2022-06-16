// Copyright 2022 Risc0, Inc.
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


use std::fs;

use password_validity_checker_methods::{PW_CHECKER_ID, PW_CHECKER_PATH};                    
use risc0_zkvm_core::Digest;
use risc0_zkvm_host::Prover;
use risc0_zkvm_serde::{from_slice, to_vec};
use tempfile::tempdir;
use rand::prelude::*;

fn main() {
    let password: &str = "S00perSecr1t!!!";

    let mut rng = StdRng::from_entropy();
    let mut salt_bytes = [0u8, 32];
    rng.fill(&mut salt_bytes);

    // a new prover is created to run the pw_checker method
    let mut prover = Prover::new(&PW_CHECKER_PATH, PW_CHECKER_ID).unwrap();

    // Adding input to the prover makes it readable by the guest
    prover
        .add_input(to_vec(&password).unwrap().as_slice())
        .unwrap();
    prover
        .add_input(to_vec(&salt_bytes).unwrap().as_slice())
        .unwrap();

    let receipt = prover.run().unwrap();
    let password_hash: Digest = from_slice(&receipt.get_journal_vec().unwrap()).unwrap();
    println!("Password hash is: {}", &password_hash);

    // In most scenarios, we would serialize and send the receipt to a verifier here
    // The verifier checks the receipt with the following call, which panics if the receipt is wrong
    receipt.verify(PW_CHECKER_ID).unwrap();
}