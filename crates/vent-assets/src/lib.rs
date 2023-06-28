pub mod mesh;
pub mod pool;
pub mod shader;

pub trait Asset {
    fn get_file_extensions() -> &'static str
    where
        Self: Sized;
}
