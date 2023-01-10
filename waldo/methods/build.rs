use std::collections::hash_map::HashMap;

use risc0_build::GuestOptions;

fn main() {
    risc0_build::embed_methods_with_options(HashMap::from([(
        "waldo-methods-guest",
        GuestOptions {
            // NOTE: The Where's Waldo method is not particularly well-optimized (it was not a
            // primary goal for this example) and so we need a larger than default code limit. This
            // increase build times, and the length of the execution trace leads to a proving time
            // that is quite long.
            code_limit: 23,
            features: vec![],
            std: true,
        },
    )]));
}
