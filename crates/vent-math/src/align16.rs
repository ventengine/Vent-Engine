#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub(crate) struct Align16<T>(pub T);

impl<T> Align16<T> {
    #[allow(dead_code)]
    pub fn as_ptr(&self) -> *const T {
        &self.0
    }

    #[allow(dead_code)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.0
    }
}
