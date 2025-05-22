use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use rhai::CustomType;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };

    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn length_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn normalize(self) -> Vec2 {
        let len = self.length();
        if len != 0.0 { self / len } else { Vec2::ZERO }
    }

    #[inline]
    pub fn distance(a: Vec2, b: Vec2) -> f32 {
        (a - b).length()
    }

    #[inline]
    pub fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
        a + (b - a) * t
    }

    #[inline]
    pub fn perp(self) -> Vec2 {
        Vec2 {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn get_x(&mut self) -> f32 {
        self.x
    }

    pub fn get_y(&mut self) -> f32 {
        self.y
    }
}

// Operator Overloads - Vec2 <op> Vec2
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul for Vec2 {
    type Output = Vec2;
    fn mul(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x * other.x, self.y * other.y)
    }
}

impl Div for Vec2 {
    type Output = Vec2;
    fn div(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x / other.x, self.y / other.y)
    }
}

// Operator Overloads - Vec2 <op> Scalar
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Vec2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, other: Vec2) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, other: Vec2) {
        self.x /= other.x;
        self.y /= other.y;
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

// Negation
impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vec2::new(-self.x, -self.y)
    }
}

// Indexing
impl Index<usize> for Vec2 {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index out of bounds!"),
        }
    }
}

impl IndexMut<usize> for Vec2 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vec2 index out of bounds!"),
        }
    }
}

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

impl CustomType for Vec2 {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("Vec2")
            .with_fn("create", Self::new)
            .with_fn::<_, 0, false, Self, false>("create", || Vec2::new(0.0, 0.0))
            .with_fn("zero", || (Vec2::ZERO))
            .with_fn("one", || (Vec2::ONE))
            .with_fn("+", vec2_add)
            .with_fn("+", vec2_add_scalar)
            .with_fn("-", vec2_neg)
            .with_fn("-", vec2_sub)
            .with_fn("*", vec2_mul)
            .with_fn("/", vec2_div)
            .with_fn("/", vec2_div_scalar)
            // with_fn <Op>Assign operators
            .with_fn("+=", vec2_add_assign)
            .with_fn("-=", vec2_sub_assign)
            .with_fn("*=", vec2_mul_assign)
            .with_fn("*=", vec2_mul_assign_scalar)
            .with_fn("/=", vec2_div_assign)
            .with_fn("/=", vec2_div_assign_scalar)
            .with_fn("lerp", Self::lerp)
            .with_fn("perp", Self::perp)
            .with_fn("distance", Self::distance)
            .with_fn("length", Self::length)
            .with_fn("length_sq", Self::length_sq)
            .with_fn("normalize", Self::normalize)
            .with_fn("dot", Self::dot)
            .with_get_set("x", Self::get_x, Self::set_x)
            .with_get_set("y", Self::get_y, Self::set_y);
    }
}
