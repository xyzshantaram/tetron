use std::rc::Rc;

use rhai::{Engine, Module};

use crate::engine::physics::vec2::Vec2;
use crate::scripting::NativeModule;

// Operator functions
fn vec2_add(a: Vec2, b: Vec2) -> Vec2 {
    a + b
}
fn vec2_sub(a: Vec2, b: Vec2) -> Vec2 {
    a - b
}
fn vec2_mul(a: Vec2, b: Vec2) -> Vec2 {
    a * b
}
fn vec2_div(a: Vec2, b: Vec2) -> Vec2 {
    a / b
}
fn vec2_add_scalar(v: Vec2, s: f32) -> Vec2 {
    v * s
}
fn vec2_div_scalar(v: Vec2, s: f32) -> Vec2 {
    v / s
}
fn vec2_neg(v: Vec2) -> Vec2 {
    -v
}

// Operator-assign functions
fn vec2_add_assign(a: &mut Vec2, b: Vec2) {
    *a += b;
}
fn vec2_sub_assign(a: &mut Vec2, b: Vec2) {
    *a -= b;
}
fn vec2_mul_assign(a: &mut Vec2, b: Vec2) {
    *a *= b;
}
fn vec2_div_assign(a: &mut Vec2, b: Vec2) {
    *a /= b;
}
fn vec2_mul_assign_scalar(a: &mut Vec2, s: f32) {
    *a *= s;
}
fn vec2_div_assign_scalar(a: &mut Vec2, s: f32) {
    *a /= s;
}

pub fn physics_module(engine: &mut Engine) -> NativeModule {
    let mut module = Module::new();

    engine
        // Binary operators
        .register_fn("+", vec2_add)
        .register_fn("+", vec2_add_scalar)
        .register_fn("-", vec2_neg)
        .register_fn("-", vec2_sub)
        .register_fn("*", vec2_mul)
        .register_fn("/", vec2_div)
        .register_fn("/", vec2_div_scalar)
        // <Op>Assign operators
        .register_fn("+=", vec2_add_assign)
        .register_fn("-=", vec2_sub_assign)
        .register_fn("*=", vec2_mul_assign)
        .register_fn("*=", vec2_mul_assign_scalar)
        .register_fn("/=", vec2_div_assign)
        .register_fn("/=", vec2_div_assign_scalar);

    module.set_sub_module("Vec2", {
        let mut sub = Module::new();
        sub.set_native_fn("create", |x, y| Ok(Vec2::new(x, y)));

        sub.set_native_fn("zero", || Ok(Vec2::ZERO));
        sub.set_native_fn("one", || Ok(Vec2::ONE));
        sub
    });

    module
        .set_custom_type::<Vec2>("Vec2")
        .set_native_fn("distance", |a: Vec2, b: Vec2| Ok(Vec2::distance(a, b)));

    module.set_native_fn("lerp", |a: Vec2, b: Vec2, t: f32| Ok(Vec2::lerp(a, b, t)));
    module.set_native_fn("perp", |a: Vec2| Ok(Vec2::perp(a)));
    module.set_native_fn("length", |a: &Vec2| Ok(Vec2::length(a)));
    module.set_native_fn("length_sq", |a: &Vec2| Ok(Vec2::length_sq(a)));
    module.set_native_fn("normalize", |a: Vec2| Ok(Vec2::normalize(a)));
    module.set_native_fn("dot", |a: Vec2, b: Vec2| Ok(Vec2::dot(a, b)));

    ("physics", Rc::new(module))
}
