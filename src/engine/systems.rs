use super::{entity::EntityRef, world::WorldRef};
use crate::{
    error::TetronError,
    utils::{Registrable, RuneString},
};
use rune::{Value, runtime::Object};
use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

#[derive(Clone, rune::Any)]
pub struct Ctx {
    #[rune(get)]
    world: WorldRef,
    #[rune(get)]
    dt: f64,
}

fn vec_str_to_hashset(v: &Value) -> Result<HashSet<String>, TetronError> {
    if let Ok(vec) = v.borrow_ref::<rune::runtime::Vec>() {
        let mut set = HashSet::<String>::new();
        for item in vec.iter() {
            if item.type_id() != TypeId::of::<RuneString>() {
                return Err(TetronError::Runtime("invalid item {item:?}".into()));
            } else {
                set.insert(item.borrow_string_ref()?.to_string());
            }
        }
        Ok(set)
    } else {
        Ok(Default::default())
    }
}

impl Ctx {
    pub fn new(world: WorldRef, dt: f64) -> Self {
        Self { world, dt }
    }

    pub fn query_with_sets(
        &self,
        tags: HashSet<String>,
        behaviours: HashSet<String>,
    ) -> Result<Vec<EntityRef>, TetronError> {
        if let Some((_, scene)) = self.world.current_scene()? {
            let entities = scene.entities();
            if tags.is_empty() && behaviours.is_empty() {
                return Ok(entities);
            }

            let result = entities
                .into_iter()
                .filter(|entity| {
                    let tags_matched = tags.is_empty() || tags.iter().any(|t| entity.has_tag(t));
                    let behaviours_matched =
                        behaviours.is_empty() || behaviours.iter().all(|b| entity.has_behaviour(b));

                    tags_matched && behaviours_matched
                })
                .collect();

            return Ok(result);
        }

        Ok(Vec::new())
    }

    #[rune::function(keep)]
    pub fn query(&self, query: Object) -> Vec<EntityRef> {
        let parse = |key| -> HashSet<String> {
            query
                .get(key)
                .map(vec_str_to_hashset)
                .transpose()
                .expect("Engine bug: failed to convert query parameter")
                .unwrap_or_default()
        };

        let tags = parse("tag");
        let behaviours = parse("b");

        self.query_with_sets(tags, behaviours)
            .expect("Engine bug: failed to execute query")
    }
}

impl Registrable for Ctx {
    fn register(module: &mut rune::Module) -> Result<(), rune::ContextError> {
        module.ty::<Ctx>()?;
        module.function_meta(Ctx::query__meta)?;
        Ok(())
    }
}
