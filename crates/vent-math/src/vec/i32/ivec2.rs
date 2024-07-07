use std::ops::{Mul, MulAssign};

#[repr(C)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    /// All zeroes.
    pub const ZERO: Self = Self::splat(0);

    /// All ones.
    pub const ONE: Self = Self::splat(1);

    /// All negative ones.
    pub const NEG_ONE: Self = Self::splat(-1);

    /// All `i32::MIN`.
    pub const MIN: Self = Self::splat(i32::MIN);

    /// All `i32::MAX`.
    pub const MAX: Self = Self::splat(i32::MAX);

    /// A unit vector pointing along the positive X axis.
    pub const X: Self = Self::new(1, 0);

    /// A unit vector pointing along the positive Y axis.
    pub const Y: Self = Self::new(0, 1);

    /// A unit vector pointing along the negative X axis.
    pub const NEG_X: Self = Self::new(-1, 0);

    /// A unit vector pointing along the negative Y axis.
    pub const NEG_Y: Self = Self::new(0, -1);

    /// The unit axes.
    pub const AXES: [Self; 2] = [Self::X, Self::Y];

    /// Creates a new vector.
    #[inline(always)]
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Creates a vector with all elements set to `v`.
    #[inline]
    #[must_use]
    pub const fn splat(v: i32) -> Self {
        Self { x: v, y: v }
    }
}

impl Mul<i32> for IVec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: i32) -> Self {
        Self {
            x: self.x.mul(rhs),
            y: self.y.mul(rhs),
        }
    }
}

impl MulAssign<i32> for IVec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        self.x.mul_assign(rhs);
        self.y.mul_assign(rhs);
    }
}

impl Mul<IVec2> for i32 {
    type Output = IVec2;
    #[inline]
    fn mul(self, rhs: IVec2) -> IVec2 {
        IVec2 {
            x: self.mul(rhs.x),
            y: self.mul(rhs.y),
        }
    }
}
