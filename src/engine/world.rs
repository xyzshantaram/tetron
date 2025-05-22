use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    TetronError,
    scripting::utils::{FnOpts, register_fn},
};

use super::scene::SceneRef;
use rhai::{
    Dynamic, Engine, EvalAltResult, FnPtr, Module, NativeCallContext, NativeCallContextStore,
    Position, plugin::RhaiResult,
};

struct WorldContext {
    world: WorldRef,
}

type WorldEventListener = (NativeCallContextStore, FnPtr);

#[derive(Default)]
pub struct World {
    scenes: HashMap<String, SceneRef>,
    current_scene: (String, SceneRef),
    listeners: HashMap<String, WorldEventListener>,
}

#[derive(Clone)]
pub struct WorldRef(Rc<RefCell<World>>);

impl WorldRef {
    fn on(&self, event: &str, listener: WorldEventListener) -> Result<(), TetronError> {
        self.0
            .try_borrow_mut()
            .map_err(|e| {
                TetronError::RhaiRuntime(
                    format!("Could not setup event listener {event}: {e}"),
                    None,
                )
            })?
            .listeners
            .insert(event.into(), listener);
        Ok(())
    }

    fn insert(&self, name: &str, scene: SceneRef) -> RhaiResult {
        self.0
            .try_borrow_mut()
            .map_err(|e| {
                TetronError::RhaiRuntime(format!("Could not load scene \"{name}\": {e}"), None)
            })?
            .scenes
            .insert(name.into(), scene);

        Ok(Dynamic::UNIT)
    }

    fn load_scene(&self, name: &str) -> RhaiResult {
        let scene = self
            .0
            .try_borrow()
            .map_err(|e| {
                TetronError::RhaiRuntime(format!("Could not load scene \"{name}\": {e}"), None)
            })?
            .scenes
            .get(name)
            .cloned()
            .ok_or_else(|| {
                TetronError::RhaiRuntime(
                    format!("Could not load scene \"{name}\": Not found"),
                    None,
                )
            })?;

        self.0
            .try_borrow_mut()
            .map_err(|e| {
                TetronError::RhaiRuntime(format!("Could not load scene \"{name}\": {e}"), None)
            })?
            .current_scene = (name.to_owned(), scene);

        Ok(Dynamic::UNIT)
    }
}

impl World {
    fn create() -> WorldRef {
        WorldRef(Rc::new(RefCell::new(Self::default())))
    }

    pub fn register(engine: &mut Engine, module: &mut Module) {
        module.set_custom_type::<WorldRef>("World");
        register_fn(
            module,
            "load_scene",
            WorldRef::load_scene,
            &FnOpts::default(),
        );

        register_fn(module, "insert", WorldRef::insert, &FnOpts::default());

        engine.register_raw_fn(
            "on",
            [
                std::any::TypeId::of::<&mut WorldRef>(),
                std::any::TypeId::of::<&str>(),
                std::any::TypeId::of::<FnPtr>(),
            ],
            |context: NativeCallContext, args: &mut [&mut Dynamic]| {
                let fp: FnPtr = args[2].take().cast::<FnPtr>(); // 3rd argument - function pointer
                let value: &str = args[1].take().cast::<&str>(); // 2nd argument - event type
                let this_ptr = args
                    .get_mut(0)
                    .ok_or(EvalAltResult::ErrorUnboundThis(Position::NONE))?
                    .clone()
                    .try_cast_result::<WorldRef>()
                    .map_err(|e| {
                        EvalAltResult::ErrorRuntime(
                            format!("Expected WorldRef, found {e}").into(),
                            Position::NONE,
                        )
                    })?;

                this_ptr.on(value, (context.store_data(), fp));
                Ok(())
            },
        );
        module.set_sub_module("World", {
            let mut sub = Module::new();
            sub.set_native_fn("create", || Ok(Self::create()));
            sub
        });
    }

    fn game_loop() {}
}
