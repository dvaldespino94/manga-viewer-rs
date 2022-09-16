use std::{ffi::CString, path::Path};

use crate::unarr::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArEntryInfo<'a> {
    pub name: &'a str,
    pub offset: i64,
    pub size: usize,
    pub filetime: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct Archive {
    pub handle: *mut ArArchive,
    pub stream: *mut ArStream,
}

#[allow(unused)]
impl Archive {
    pub fn new(_path: &str) -> Result<Self, String> {
        let c_path = CString::new(_path).unwrap();
        let path = Path::new(_path);

        let stream = unsafe { ar_open_file(c_path.as_ptr()) };

        let extension = path.extension().unwrap().to_str().unwrap();

        match extension {
            "rar" | "cbr" => {
                return Ok(Archive {
                    handle: unsafe { ar_open_rar_archive(stream) },
                    stream,
                });
            }
            "zip" | "cbz" => {
                return Ok(Archive {
                    handle: unsafe { ar_open_zip_archive(stream, false) },
                    stream,
                });
            }
            _ => {}
        }

        return Err(format!("Not supported extension: '{extension}'"));
    }

    pub fn close(&self) {
        unsafe { ar_close_archive(self.handle) };
        unsafe { ar_close(self.stream) };
    }

    pub fn read(&self, offset: i64, size: usize) -> Result<Vec<u8>, String> {
        if unsafe { ar_parse_entry_at(self.handle, offset) } {
            let mut buffer = Vec::<u8>::with_capacity(size);
            let buffer_ptr = buffer.as_mut_ptr();

            if unsafe {
                ar_entry_uncompress(
                    self.handle,
                    buffer_ptr as *mut std::ffi::c_void,
                    size.try_into().unwrap(),
                )
            } {
                unsafe { buffer.set_len(size) };

                return Ok(buffer);
            } else {
                return Err("Error uncompresing".to_string());
            }
        } else {
            return Err("Error parsing entry".to_string());
        }
    }
}

impl Iterator for Archive {
    type Item = ArEntryInfo<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { ar_parse_entry(self.handle) } {
            let name_source = unsafe { ar_entry_get_name(self.handle) };
            let name_len = unsafe { libc::strlen(name_source) };
            let filetime = unsafe { ar_entry_get_filetime(self.handle) };
            let name = unsafe { libc::malloc(name_len + 1) as *mut i8 };

            unsafe {
                libc::strcpy(name, name_source);
            }

            return Some(ArEntryInfo {
                name: unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap(),
                filetime,
                offset: unsafe { ar_entry_get_offset(self.handle) },
                size: unsafe { ar_entry_get_size(self.handle).try_into().unwrap() },
            });
        } else {
            self.close();
            return None;
        }
    }
}

// #[allow(unused)]
// fn test() {
//     let default_document_path =
//         "/Users/david.valdespino/Downloads/Spy x Family/Spy x Family - Tomo 02 (#006-011).cbr";
//
//     let archive = Archive::new(default_document_path);
//
//     for entry in archive {
//         let _data = entry.read(archive.handle).expect("Error reading data");
//         if _data.len() == entry.size.try_into().unwrap() {
//             eprintln!("Size matches: {}", _data.len());
//         } else {
//             eprintln!("{} != {}", _data.len(), entry.size);
//         }
//     }
// }
