use std::{
    arch::x86_64::{__m128, _mm_loadu_ps, _mm_mul_ps, _mm_set1_ps, _mm_store_ps},
    ops::{Deref, DerefMut, Mul, MulAssign},
};

use crate::align16::Align16;

use super::{vec2::Vec2, vec3::Vec3};

#[repr(C)]
union UnionCast {
    a: [f32; 4],
    v: Vec4,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vec4(pub(crate) __m128);

impl Vec4 {
    pub const ZERO: Self = Self::splat(0.0);

    pub const ONE: Self = Self::splat(1.0);

    pub const NEG_ONE: Self = Self::splat(-1.0);

    pub const MIN: Self = Self::splat(f32::MIN);

    pub const MAX: Self = Self::splat(f32::MAX);

    pub const NAN: Self = Self::splat(f32::NAN);

    pub const INFINITY: Self = Self::splat(f32::INFINITY);

    pub const NEG_INFINITY: Self = Self::splat(f32::NEG_INFINITY);

    pub const X: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    pub const Y: Self = Self::new(0.0, 1.0, 0.0, 0.0);

    pub const Z: Self = Self::new(0.0, 0.0, 1.0, 0.0);

    pub const W: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const NEG_X: Self = Self::new(-1.0, 0.0, 0.0, 0.0);

    pub const NEG_Y: Self = Self::new(0.0, -1.0, 0.0, 0.0);

    pub const NEG_Z: Self = Self::new(0.0, 0.0, -1.0, 0.0);

    pub const NEG_W: Self = Self::new(0.0, 0.0, 0.0, -1.0);

    pub const AXES: [Self; 4] = [Self::X, Self::Y, Self::Z, Self::W];

    #[inline(always)]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        unsafe { UnionCast { a: [x, y, z, w] }.v }
    }

    #[inline]
    #[must_use]
    pub const fn from_array(a: [f32; 4]) -> Self {
        Self::new(a[0], a[1], a[2], a[3])
    }

    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        unsafe { UnionCast { a: [v; 4] }.v }
    }
}

impl Mul<Vec4> for Vec4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(unsafe { _mm_mul_ps(self.0, rhs.0) })
    }
}

impl MulAssign<Vec4> for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = unsafe { _mm_mul_ps(self.0, rhs.0) };
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self(unsafe { _mm_mul_ps(self.0, _mm_set1_ps(rhs)) })
    }
}

impl MulAssign<f32> for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.0 = unsafe { _mm_mul_ps(self.0, _mm_set1_ps(rhs)) };
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;
    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4(unsafe { _mm_mul_ps(_mm_set1_ps(self), rhs.0) })
    }
}

impl From<Vec4> for __m128 {
    #[inline(always)]
    fn from(t: Vec4) -> Self {
        t.0
    }
}

impl From<__m128> for Vec4 {
    #[inline(always)]
    fn from(t: __m128) -> Self {
        Self(t)
    }
}

impl From<[f32; 4]> for Vec4 {
    #[inline]
    fn from(a: [f32; 4]) -> Self {
        Self(unsafe { _mm_loadu_ps(a.as_ptr()) })
    }
}

impl From<Vec4> for [f32; 4] {
    #[inline]
    fn from(v: Vec4) -> Self {
        use core::mem::MaybeUninit;
        let mut out: MaybeUninit<Align16<Self>> = MaybeUninit::uninit();
        unsafe {
            _mm_store_ps(out.as_mut_ptr().cast(), v.0);
            out.assume_init().0
        }
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    #[inline]
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Self::new(t.0, t.1, t.2, t.3)
    }
}

impl From<Vec4> for (f32, f32, f32, f32) {
    #[inline]
    fn from(v: Vec4) -> Self {
        use core::mem::MaybeUninit;
        let mut out: MaybeUninit<Align16<Self>> = MaybeUninit::uninit();
        unsafe {
            _mm_store_ps(out.as_mut_ptr().cast(), v.0);
            out.assume_init().0
        }
    }
}

impl From<(Vec3, f32)> for Vec4 {
    #[inline]
    fn from((v, w): (Vec3, f32)) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }
}

impl From<(f32, Vec3)> for Vec4 {
    #[inline]
    fn from((x, v): (f32, Vec3)) -> Self {
        Self::new(x, v.x, v.y, v.z)
    }
}

impl From<(Vec2, f32, f32)> for Vec4 {
    #[inline]
    fn from((v, z, w): (Vec2, f32, f32)) -> Self {
        Self::new(v.x, v.y, z, w)
    }
}

impl From<(Vec2, Vec2)> for Vec4 {
    #[inline]
    fn from((v, u): (Vec2, Vec2)) -> Self {
        Self::new(v.x, v.y, u.x, u.y)
    }
}

impl Deref for Vec4 {
    type Target = crate::deref::Vec4<f32>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Self).cast() }
    }
}

impl DerefMut for Vec4 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self).cast() }
    }
}
