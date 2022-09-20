use std::io;

#[cfg(windows)]
extern crate windres;
use windres::Build;

fn main() -> io::Result<()> {
    // println!("cargo:rustc-link-lib=unarr");
    Build::new().compile("resources.rc").unwrap();
    Ok(())
}
