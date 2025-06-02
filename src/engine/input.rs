use rune::{ContextError, Module, docstring};
use sdl2::{event::Event, keyboard::Scancode};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use crate::error::TetronError;

#[derive(Default)]
pub struct KeyState {
    down: HashSet<Scancode>,
    released: HashSet<Scancode>,
    pressed: HashSet<Scancode>,
    held: HashSet<Scancode>,
}

impl KeyState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::KeyDown {
                scancode: Some(sc),
                repeat: false,
                ..
            } => {
                self.down.insert(*sc);
                self.pressed.insert(*sc);
                self.held.insert(*sc);
            }
            Event::KeyUp {
                scancode: Some(sc), ..
            } => {
                self.down.remove(sc);
                self.released.insert(*sc);
                self.held.remove(sc);
            }
            _ => {}
        }
    }

    pub fn next_frame(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn is_down(&self, name: &str) -> bool {
        self.check_set(name, &self.down)
    }

    pub fn just_released(&self, name: &str) -> bool {
        self.check_set(name, &self.released)
    }

    pub fn just_pressed(&self, name: &str) -> bool {
        self.check_set(name, &self.pressed)
    }

    pub fn is_held(&self, name: &str) -> bool {
        self.check_set(name, &self.held)
    }

    fn check_set(&self, name: &str, set: &HashSet<Scancode>) -> bool {
        Scancode::from_name(name).is_some_and(|v| set.contains(&v))
    }
}

/// Builds a Rune scripting module exposing input (keyboard) queries to scripts.
///
/// Provided functions:
/// - `is_down(key: &str) -> bool`: Returns true if the key is currently down.
/// - `just_pressed(key: &str) -> bool`: True if the key was pressed this frame.
/// - `just_released(key: &str) -> bool`: True if the key was released this frame.
/// - `is_held(key: &str) -> bool`: True if the key is held down.
///
/// # Arguments
/// * `input` - Shared state of the current keyboard inputs.
pub fn module(input: Arc<RwLock<KeyState>>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["input"])?;

    module
        .function("is_down", {
            let input = Arc::clone(&input);
            move |k: &str| -> Result<bool, TetronError> {
                let guard = input.read()?;
                Ok(guard.is_down(k))
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key is currently down (pressed).
            ///
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    module
        .function("just_pressed", {
            let input = Arc::clone(&input);
            move |k: &str| -> Result<bool, TetronError> {
                let guard = input.read()?;
                Ok(guard.just_pressed(k))
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key was pressed this frame.
            ///
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    module
        .function("just_released", {
            let input = Arc::clone(&input);
            move |k: &str| -> Result<bool, TetronError> {
                let guard = input.read()?;
                Ok(guard.just_released(k))
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key was released this frame.
            ///
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    module
        .function("is_held", {
            let input = Arc::clone(&input);
            move |k: &str| -> Result<bool, TetronError> {
                let guard = input.read()?;
                Ok(guard.is_held(k))
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key is held down (not freshly pressed, but kept down).
            ///
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    Ok(module)
}
