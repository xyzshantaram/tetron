use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

#[derive(rune::Any, Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    #[rune(get, set)]
    pub x: f64,
    #[rune(get, set)]
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };

    #[inline]
    #[rune::function(path = Self::new, keep)]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn length(&self) -> f64 {
        self.length_sq().sqrt()
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn length_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn normalize(self) -> Vec2 {
        let len = self.length();
        if len != 0.0 { self / len } else { Vec2::ZERO }
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn distance(self, b: Vec2) -> f64 {
        (self - b).length()
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn lerp(self, b: Vec2, t: f64) -> Vec2 {
        self + (b - self) * t
    }

    #[inline]
    #[rune::function(keep, instance)]
    pub fn perp(self) -> Vec2 {
        Vec2 {
            x: -self.y,
            y: self.x,
        }
    }

    #[rune::function(keep, path = Self::zero)]
    pub fn zero() -> Vec2 {
        Self::ZERO
    }

    #[rune::function(path = Self::one)]
    pub fn one() -> Vec2 {
        Self::ZERO
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
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f64> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f64) -> Vec2 {
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

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
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

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
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
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index out of bounds!"),
        }
    }
}

impl IndexMut<usize> for Vec2 {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vec2 index out of bounds!"),
        }
    }
}
