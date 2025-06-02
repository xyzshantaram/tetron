use tetron::{engine::input::KeyState, scripting};

use scripting::tetron_context;
use std::sync::{Arc, RwLock};
use stupid_simple_kv::{Kv, MemoryBackend};

pub fn main() {
    rune::cli::Entry::new()
    .about(format_args!("tetron cli. this is not meant to be used in a terminal - it is simply for the language server"))
    .context(&mut |_opts| {
        let backends = (
            Box::new(MemoryBackend::new()),
            Box::new(MemoryBackend::new()),
        );
        let flags = Arc::new(RwLock::new(Kv::new(backends.0)));
        let config = Arc::new(Kv::new(backends.1));
        let input = Arc::new(RwLock::new(KeyState::new()));
        Ok(tetron_context(flags.clone(), config.clone(), input.clone()).expect("Error building tetron context"))
    })
    .run();
}
