use std::env;

use anyhow::Result;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    let out_dir = env::var("OUT_DIR")?;
    let copy_options = CopyOptions::new().overwrite(true);
    copy_items(&["res/"], out_dir, &copy_options)?;
    

    Ok(())
}
