use std::collections::hash_map::HashMap;
use risc0_build::GuestOptions;

fn main() {
    risc0_build::embed_methods_with_options(HashMap::from([
        (
            "waldo-methods-guest",
            GuestOptions {
                code_limit: 23,
                features: vec![],
                std: true,
            }
        )
    ]));
}
