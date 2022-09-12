use std::path::Path;

use crate::traits::IChunkProvider;

pub struct DirChunkProvider {
    files: Vec<String>,
}

impl IChunkProvider for DirChunkProvider {
    fn new() -> Self {
        Self::default()
    }

    fn get_chunk(&self, _index: usize) -> Option<crate::structs::Chunk> {
        None
    }

    fn chunk_count(&self) -> usize {
        0
    }

    fn done_processing(&self) -> bool {
        false
    }

    fn destroy(&self) {
        todo!()
    }

    fn open(self: &mut DirChunkProvider, _path: &str) -> bool {
        let path = Path::new(_path);
        if path.exists() && path.is_dir() {
            if let Ok(dir) = path.read_dir() {
                self.files = dir
                    .map(|element| element.unwrap().path().to_str().unwrap().to_string())
                    .filter(|element| element.ends_with(".jpg") || element.ends_with(".png"))
                    .collect();
            }
            println!("Got files: {:?}", self.files);
            return true;
        }

        return false;
    }

    fn get_image(&self, _index: usize) -> Option<&raylib::texture::Image> {
        None
    }

    fn get_metadata(_path: &str) -> Option<crate::structs::ComicMetadata> {
        None
    }
}

impl Default for DirChunkProvider {
    fn default() -> Self {
        Self { files: Vec::new() }
    }
}
