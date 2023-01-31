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

use std::collections::hash_map::HashMap;

use risc0_build::GuestOptions;

fn main() {
    risc0_build::embed_methods_with_options(HashMap::from([(
        "waldo-methods-guest",
        GuestOptions {
            // NOTE: The Where's Waldo method is not particularly well-optimized (it was not a
            // primary goal for this example) and so we need a larger than default code limit. This
            // increases build times, and the length of the execution trace leads to a proving time
            // that is quite long.
            code_limit: 23,
            features: vec![],
            std: true,
        },
    )]));
}
