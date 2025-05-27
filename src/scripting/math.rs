use std::{
    f32::consts,
    ops::{Add, Div, Mul, Sub},
};

use rune::{ContextError, Module, docstring, runtime::Protocol};

use crate::engine::physics::vec2::Vec2;

#[rune::function]
fn sin(x: f32) -> f32 {
    x.sin()
}

#[rune::function]
fn cos(x: f32) -> f32 {
    x.cos()
}

#[rune::function]
fn tan(x: f32) -> f32 {
    x.tan()
}

#[rune::function]
fn asin(x: f32) -> f32 {
    x.asin()
}

#[rune::function]
fn acos(x: f32) -> f32 {
    x.acos()
}

#[rune::function]
fn atan(x: f32) -> f32 {
    x.atan()
}

#[rune::function]
fn atan2(y: f32, x: f32) -> f32 {
    y.atan2(x)
}

#[rune::function]
fn sqrt(x: f32) -> f32 {
    x.sqrt()
}

#[rune::function]
fn abs(x: f32) -> f32 {
    x.abs()
}

#[rune::function]
fn signum(x: f32) -> f32 {
    x.signum()
}

#[rune::function]
fn min(a: f32, b: f32) -> f32 {
    a.min(b)
}

#[rune::function]
fn max(a: f32, b: f32) -> f32 {
    a.max(b)
}

#[rune::function]
fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.min(max).max(min)
}

#[rune::function]
fn pow(x: f32, y: f32) -> f32 {
    x.powf(y)
}

#[rune::function]
fn exp(x: f32) -> f32 {
    x.exp()
}

#[rune::function]
fn ln(x: f32) -> f32 {
    x.ln()
}

#[rune::function]
fn floor(x: f32) -> f32 {
    x.floor()
}

#[rune::function]
fn ceil(x: f32) -> f32 {
    x.ceil()
}

#[rune::function]
fn round(x: f32) -> f32 {
    x.round()
}

#[rune::function]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

impl Vec2 {
    #[rune::function(instance, protocol = ADD_ASSIGN)]
    fn add_assign_rune(&mut self, rhs: Vec2) {
        *self += rhs;
    }

    #[rune::function(instance, protocol = SUB_ASSIGN)]
    fn sub_assign_rune(&mut self, rhs: Vec2) {
        *self -= rhs;
    }

    #[rune::function(instance, protocol = MUL_ASSIGN)]
    fn mul_assign_rune(&mut self, rhs: Vec2) {
        *self *= rhs;
    }

    #[rune::function(instance, protocol = DIV_ASSIGN)]
    fn div_assign_rune(&mut self, rhs: Vec2) {
        *self /= rhs;
    }

    #[rune::function(instance, protocol = DIV)]
    fn div_rune(self, rhs: Vec2) -> Vec2 {
        self / rhs
    }

    #[rune::function(instance, protocol = MUL)]
    fn mul_rune(self, rhs: Vec2) -> Vec2 {
        self * rhs
    }

    #[rune::function(instance, protocol = SUB)]
    fn sub_rune(self, rhs: Vec2) -> Vec2 {
        self - rhs
    }

    #[rune::function(instance, protocol = ADD)]
    fn add_rune(self, rhs: Vec2) -> Vec2 {
        self + rhs
    }

    #[rune::function(instance, protocol = PARTIAL_EQ)]
    fn partial_eq_rune(&self, rhs: &Vec2) -> bool {
        self == rhs
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["math"])?;

    module
        .constant("PI", consts::PI)
        .build()?
        .docs(docstring! {
            /// Archimedes' constant, the ratio of a circle's circumference to its diameter.
        })?;

    module
        .constant("TAU", consts::TAU)
        .build()?
        .docs(docstring! {
            /// τ = 2π, the ratio of a circle's circumference to its radius.
        })?;

    module.constant("E", consts::E).build()?.docs(docstring! {
        /// Euler's number, the base of natural logarithms.
    })?;

    module
        .constant("TRIG_PI_2", consts::FRAC_PI_2)
        .build()?
        .docs(docstring! {
            /// π/2
        })?;

    module
        .constant("TRIG_PI_3", consts::FRAC_PI_3)
        .build()?
        .docs(docstring! {
            /// π/3
        })?;

    module
        .constant("TRIG_PI_4", consts::FRAC_PI_4)
        .build()?
        .docs(docstring! {
            /// π/4
        })?;

    module
        .constant("TRIG_1_PI", consts::FRAC_1_PI)
        .build()?
        .docs(docstring! {
            /// 1/π
        })?;

    module
        .constant("TRIG_2_PI", consts::FRAC_2_PI)
        .build()?
        .docs(docstring! {
            /// 2/π
        })?;

    module
        .constant("LN_2", consts::LN_2)
        .build()?
        .docs(docstring! {
            /// Natural logarithm of 2
        })?;

    module
        .constant("LN_10", consts::LN_10)
        .build()?
        .docs(docstring! {
            /// Natural logarithm of 10
        })?;

    module
        .constant("SQRT_2", consts::SQRT_2)
        .build()?
        .docs(docstring! {
            /// Square root of 2
        })?;

    module
        .constant("INV_SQRT_2", consts::FRAC_1_SQRT_2)
        .build()?
        .docs(docstring! {
            /// 1 / √2
        })?;

    module.function_meta(sin)?;
    module.function_meta(cos)?;
    module.function_meta(tan)?;
    module.function_meta(asin)?;
    module.function_meta(acos)?;
    module.function_meta(atan)?;
    module.function_meta(atan2)?;
    module.function_meta(sqrt)?;
    module.function_meta(abs)?;
    module.function_meta(signum)?;
    module.function_meta(min)?;
    module.function_meta(max)?;
    module.function_meta(clamp)?;
    module.function_meta(pow)?;
    module.function_meta(exp)?;
    module.function_meta(ln)?;
    module.function_meta(floor)?;
    module.function_meta(ceil)?;
    module.function_meta(round)?;
    module.function_meta(lerp)?;

    module.ty::<Vec2>()?;
    module.associated_function::<&rune::runtime::Protocol, _, (Vec2, Vec2), _>(
        &Protocol::ADD,
        Vec2::add,
    )?;
    module.associated_function::<&rune::runtime::Protocol, _, (Vec2, Vec2), _>(
        &Protocol::SUB,
        Vec2::sub,
    )?;
    module.associated_function::<&rune::runtime::Protocol, _, (Vec2, Vec2), _>(
        &Protocol::DIV,
        Vec2::div,
    )?;
    module.associated_function::<&rune::runtime::Protocol, _, (Vec2, Vec2), _>(
        &Protocol::MUL,
        Vec2::mul,
    )?;

    Ok(module)
}
