fn main() {
    let options_map = std::collections::HashMap::from([("methods", risc0_build::GuestOptions{ code_limit:20, features: vec![], std: false }),
                                                       ("compile", risc0_build::GuestOptions{ code_limit: 20, features: vec![], std: false }),
                                                       ("methods-guest", risc0_build::GuestOptions{ code_limit: 20, features: vec![], std: false }),
                                                       ]);
    risc0_build::embed_methods_with_options(options_map);
}
