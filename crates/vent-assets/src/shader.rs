use crate::pool::Asset;

struct Shader {}

impl Shader {}

impl Asset for Shader {
    fn get_file_extensions() -> &'static str {
        ""
    }
}
