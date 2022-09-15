use std::io;

#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    // println!("cargo:rustc-link-lib=unarr");

    #[cfg(target_os = "windows")] 
    {
        let mut res = WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }

    Ok(())
}
