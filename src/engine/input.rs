use rune::{ContextError, Module, docstring};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug)]
pub struct KeyState {
    down: HashSet<Scancode>,
    pressed: HashSet<Scancode>,
    released: HashSet<Scancode>,
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
                if !self.down.contains(sc) {
                    self.pressed.insert(*sc);
                }
                self.down.insert(*sc);
            }
            Event::KeyUp {
                scancode: Some(sc), ..
            } => {
                self.down.remove(sc);
                self.released.insert(*sc);
            }
            Event::Window {
                win_event: WindowEvent::FocusLost,
                ..
            } => {
                self.clear_all();
            }
            _ => {}
        }
    }

    pub fn next_frame(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    fn clear_all(&mut self) {
        self.down.clear();
        self.pressed.clear();
        self.released.clear();
    }

    pub fn is_down(&self, name: &str) -> bool {
        self.check_set(name, &self.down)
    }

    pub fn just_pressed(&self, name: &str) -> bool {
        self.check_set(name, &self.pressed)
    }

    pub fn just_released(&self, name: &str) -> bool {
        self.check_set(name, &self.released)
    }

    pub fn is_held(&self, name: &str) -> bool {
        self.check_set(name, &self.down) // `down` reflects held keys
    }

    fn check_set(&self, name: &str, set: &HashSet<Scancode>) -> bool {
        Scancode::from_name(name).is_some_and(|v| set.contains(&v))
    }
}

pub fn module(input: Arc<RwLock<KeyState>>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["input"])?;

    module
        .function("is_down", {
            let input = input.clone();
            move |k: &str| -> bool {
                let guard = input.read().expect("Engine bug: input lock poisoned");
                guard.is_down(k)
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key is currently down (pressed).
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    module
        .function("just_pressed", {
            let input = input.clone();
            move |k: &str| -> bool {
                let guard = input.read().expect("Engine bug: input lock poisoned");
                guard.just_pressed(k)
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key was pressed this frame.
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    module
        .function("just_released", {
            let input = input.clone();
            move |k: &str| -> bool {
                let guard = input.read().expect("Engine bug: input lock poisoned");
                guard.just_released(k)
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key was released this frame.
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    module
        .function("is_held", {
            let input = input.clone();
            move |k: &str| -> bool {
                let guard = input.read().expect("Engine bug: input lock poisoned");
                guard.is_held(k)
            }
        })
        .build()?
        .docs(docstring! {
            /// Returns true if the specified key is held down.
            /// # Arguments
            /// * `key` - The name of the key to check, as string.
        })?;

    Ok(module)
}
