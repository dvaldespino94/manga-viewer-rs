use std::ffi::CStr;
use std::ffi::CString;

use crate::unarr::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArEntryInfo<'a> {
    pub name: &'a str,
    pub offset: i64,
    pub size: usize,
    pub filetime: i64,
}

impl ArEntryInfo<'_> {
    pub fn read(&self, handle: *mut ArArchive) -> Result<Vec<u8>, String> {
        if unsafe { ar_parse_entry_at(handle, self.offset) } {
            let mut buffer = Vec::<u8>::with_capacity(self.size);
            let buffer_ptr = buffer.as_mut_ptr();

            if unsafe {
                ar_entry_uncompress(
                    handle,
                    buffer_ptr as *mut std::ffi::c_void,
                    self.size.try_into().unwrap(),
                )
            } {
                unsafe { buffer.set_len(self.size) };
                return Ok(buffer);
            } else {
                return Err("Error uncompresing".to_string());
            }
        } else {
            return Err("Error parsing entry".to_string());
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Archive {
    pub handle: *mut ArArchive,
    pub stream: *mut ArStream,
}

impl Archive {
    pub fn new(_path: &str) -> Self {
        let path = CString::new(_path).unwrap();

        let stream = unsafe { ar_open_file(path.as_ptr()) };

        //TODO: Handle different compression formats
        let handle = unsafe { ar_open_rar_archive(stream) };

        return Archive {
            handle: handle,
            stream: stream,
        };
    }

    pub fn close(&self) {
        unsafe { ar_close_archive(self.handle) };
        unsafe { ar_close(self.stream) };
    }
}

impl Iterator for Archive {
    type Item = ArEntryInfo<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        println!("Parsing entry");
        if unsafe { ar_parse_entry(self.handle) } {
            let name = unsafe { ar_entry_get_name(self.handle) };
            let filetime = unsafe { ar_entry_get_filetime(self.handle) };

            return Some(ArEntryInfo {
                name: unsafe { CStr::from_ptr(name).to_str().unwrap() },
                filetime: filetime,
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
//             println!("Size matches: {}", _data.len());
//         } else {
//             println!("{} != {}", _data.len(), entry.size);
//         }
//     }
// }
