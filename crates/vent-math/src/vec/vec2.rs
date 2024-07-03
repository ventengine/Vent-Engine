use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::splat(0.0);

    pub const ONE: Self = Self::splat(1.0);

    pub const NEG_ONE: Self = Self::splat(-1.0);

    pub const MIN: Self = Self::splat(f32::MIN);

    pub const MAX: Self = Self::splat(f32::MAX);

    pub const NAN: Self = Self::splat(f32::NAN);

    pub const INFINITY: Self = Self::splat(f32::INFINITY);

    pub const NEG_INFINITY: Self = Self::splat(f32::NEG_INFINITY);

    pub const X: Self = Self::new(1.0, 0.0);

    pub const Y: Self = Self::new(0.0, 1.0);

    pub const NEG_X: Self = Self::new(-1.0, 0.0);

    pub const NEG_Y: Self = Self::new(0.0, -1.0);

    pub const AXES: [Self; 2] = [Self::X, Self::Y];

    #[inline(always)]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        Self {
            x: self.x.div(rhs.x),
            y: self.y.div(rhs.y),
        }
    }
}

impl DivAssign<Vec2> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x.div_assign(rhs.x);
        self.y.div_assign(rhs.y);
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x.div(rhs),
            y: self.y.div(rhs),
        }
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x.div_assign(rhs);
        self.y.div_assign(rhs);
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.div(rhs.x),
            y: self.div(rhs.y),
        }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x.mul(rhs.x),
            y: self.y.mul(rhs.y),
        }
    }
}

impl MulAssign<Vec2> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x.mul_assign(rhs.x);
        self.y.mul_assign(rhs.y);
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x.mul(rhs),
            y: self.y.mul(rhs),
        }
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x.mul_assign(rhs);
        self.y.mul_assign(rhs);
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.mul(rhs.x),
            y: self.mul(rhs.y),
        }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x.add(rhs.x),
            y: self.y.add(rhs.y),
        }
    }
}

impl AddAssign<Vec2> for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x.add_assign(rhs.x);
        self.y.add_assign(rhs.y);
    }
}

impl Add<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self {
        Self {
            x: self.x.add(rhs),
            y: self.y.add(rhs),
        }
    }
}

impl AddAssign<f32> for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.x.add_assign(rhs);
        self.y.add_assign(rhs);
    }
}

impl Add<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.add(rhs.x),
            y: self.add(rhs.y),
        }
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x.sub(rhs.x),
            y: self.y.sub(rhs.y),
        }
    }
}

impl SubAssign<Vec2> for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x.sub_assign(rhs.x);
        self.y.sub_assign(rhs.y);
    }
}

impl Sub<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self {
        Self {
            x: self.x.sub(rhs),
            y: self.y.sub(rhs),
        }
    }
}

impl SubAssign<f32> for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.x.sub_assign(rhs);
        self.y.sub_assign(rhs);
    }
}

impl Sub<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.sub(rhs.x),
            y: self.sub(rhs.y),
        }
    }
}

impl Rem<Vec2> for Vec2 {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self {
            x: self.x.rem(rhs.x),
            y: self.y.rem(rhs.y),
        }
    }
}

impl RemAssign<Vec2> for Vec2 {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.x.rem_assign(rhs.x);
        self.y.rem_assign(rhs.y);
    }
}

impl Rem<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: f32) -> Self {
        Self {
            x: self.x.rem(rhs),
            y: self.y.rem(rhs),
        }
    }
}

impl RemAssign<f32> for Vec2 {
    #[inline]
    fn rem_assign(&mut self, rhs: f32) {
        self.x.rem_assign(rhs);
        self.y.rem_assign(rhs);
    }
}

impl Rem<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn rem(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.rem(rhs.x),
            y: self.rem(rhs.y),
        }
    }
}
