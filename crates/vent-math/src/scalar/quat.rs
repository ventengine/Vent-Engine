use std::{
    arch::x86_64::__m128,
    ops::{Deref, DerefMut},
};

use crate::vec::{vec3::Vec3, vec4::Vec4};

#[repr(C)]
union UnionCast {
    a: [f32; 4],
    v: Quat,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Quat(pub(crate) __m128);

impl Quat {
    const ZERO: Self = Self::from_array([0.0; 4]);

    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const NAN: Self = Self::from_array([f32::NAN; 4]);

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
    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[inline]
    #[must_use]
    pub fn from_scaled_axis(v: Vec3) -> Self {
        let length = v.length();
        if length == 0.0 {
            Self::IDENTITY
        } else {
            Self::from_axis_angle(v / length, length)
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let (s, c) = (angle * 0.5).sin_cos();
        let v = axis * s;
        Self::new(v.x, v.y, v.z, c)
    }
}

impl From<Quat> for Vec4 {
    #[inline]
    fn from(q: Quat) -> Self {
        Self(q.0)
    }
}

impl From<Quat> for (f32, f32, f32, f32) {
    #[inline]
    fn from(q: Quat) -> Self {
        Vec4::from(q).into()
    }
}

impl From<Quat> for [f32; 4] {
    #[inline]
    fn from(q: Quat) -> Self {
        Vec4::from(q).into()
    }
}

impl From<Quat> for __m128 {
    #[inline]
    fn from(q: Quat) -> Self {
        q.0
    }
}

impl Deref for Quat {
    type Target = crate::deref::Vec4<f32>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Self).cast() }
    }
}

impl DerefMut for Quat {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self).cast() }
    }
}
