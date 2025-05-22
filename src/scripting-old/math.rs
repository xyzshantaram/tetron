use std::f32::consts;
use std::rc::Rc;

use crate::scripting::NativeModule;
use rhai::Module;

pub fn math_module() -> NativeModule {
    let mut module = Module::new();

    // Constants
    module.set_var("PI", consts::PI); // π
    module.set_var("TAU", consts::TAU); // τ = 2π
    module.set_var("E", consts::E); // Euler's number
    module.set_var("TRIG_PI_2", consts::FRAC_PI_2); // π/2
    module.set_var("TRIG_PI_3", consts::FRAC_PI_3); // π/3
    module.set_var("TRIG_PI_4", consts::FRAC_PI_4); // π/4
    module.set_var("TRIG_1_PI", consts::FRAC_1_PI); // 1/π
    module.set_var("TRIG_2_PI", consts::FRAC_2_PI); // 2/π
    module.set_var("LN_2", consts::LN_2); // ln(2)
    module.set_var("LN_10", consts::LN_10); // ln(10)
    module.set_var("SQRT_2", consts::SQRT_2); // sqrt(2)
    module.set_var("INV_SQRT_2", consts::FRAC_1_SQRT_2); // 1/sqrt(2)

    // Basic math functions
    module.set_native_fn("sin", |x: f32| Ok(x.sin()));
    module.set_native_fn("cos", |x: f32| Ok(x.cos()));
    module.set_native_fn("tan", |x: f32| Ok(x.tan()));
    module.set_native_fn("asin", |x: f32| Ok(x.asin()));
    module.set_native_fn("acos", |x: f32| Ok(x.acos()));
    module.set_native_fn("atan", |x: f32| Ok(x.atan()));
    module.set_native_fn("atan2", |y: f32, x: f32| Ok(y.atan2(x)));
    module.set_native_fn("sqrt", |x: f32| Ok(x.sqrt()));
    module.set_native_fn("abs", |x: f32| Ok(x.abs()));
    module.set_native_fn("signum", |x: f32| Ok(x.signum()));
    module.set_native_fn("min", |a: f32, b: f32| Ok(a.min(b)));
    module.set_native_fn("max", |a: f32, b: f32| Ok(a.max(b)));
    module.set_native_fn("clamp", |x: f32, min: f32, max: f32| {
        Ok(x.min(max).max(min))
    });
    module.set_native_fn("pow", |x: f32, y: f32| Ok(x.powf(y)));
    module.set_native_fn("exp", |x: f32| Ok(x.exp()));
    module.set_native_fn("ln", |x: f32| Ok(x.ln()));
    module.set_native_fn("floor", |x: f32| Ok(x.floor()));
    module.set_native_fn("ceil", |x: f32| Ok(x.ceil()));
    module.set_native_fn("round", |x: f32| Ok(x.round()));

    // Linear interpolation
    module.set_native_fn("lerp", |a: f32, b: f32, t: f32| Ok((1.0 - t) * a + t * b));

    ("math", Rc::new(module))
}
