use std::io;

#[cfg(windows)]
extern crate windres;
#[cfg(windows)]
use windres::Build;

fn main() -> io::Result<()> {
    // println!("cargo:rustc-link-lib=unarr");
    #[cfg(windows)]
    Build::new().compile("resources.rc").unwrap();
    Ok(())
}
