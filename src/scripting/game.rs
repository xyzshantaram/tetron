use super::NativeModule;
use crate::engine::{entity::Entity, world::World};
use rhai::{Engine, Module};
use std::rc::Rc;

pub fn game_module(engine: &mut Engine) -> NativeModule {
    let mut module = Module::new();
    World::register(engine, &mut module);
    Entity::register(&mut module);

    ("game", Rc::new(module))
}
