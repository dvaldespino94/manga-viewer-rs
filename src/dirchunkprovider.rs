use crate::processing::get_chunks_from_image;
use raylib::prelude::*;
use std::{cmp::max, collections::HashMap, path::Path};

use crate::{structs::Chunk, traits::IChunkProvider};

pub struct DirChunkProvider {
    files: Vec<String>,
    chunks: Vec<Chunk>,
    images: HashMap<usize, Image>,
    image_loading_order: Vec<usize>,
    last_loaded_image: i32,
}

impl DirChunkProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IChunkProvider for DirChunkProvider {
    fn get_chunk(&mut self, index: usize) -> Option<&crate::structs::Chunk> {
        if index >= self.chunks.len() {
            self.last_loaded_image += 1;
            self.get_image(self.last_loaded_image as usize);
        }

        self.chunks.get(index)
    }

    fn chunk_count(&self) -> usize {
        if self.files.len() > 0 {
            return max(self.chunks.len() + 1, 1);
        }

        return 0;
    }

    fn done_processing(&self) -> bool {
        self.last_loaded_image >= self.files.len() as i32
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

            //Preload first image
            self.get_image(0);

            return true;
        }

        return false;
    }

    fn get_image(&mut self, index: usize) -> Option<&raylib::texture::Image> {
        println!("Getting image {}", index);

        if index >= self.files.len() {
            return None;
        }

        if self.images.contains_key(&index) {
            return self.images.get(&index);
        }

        println!("Image {} not found, fetching...", index);

        let mut image = Image::load_image(self.files[index].as_str()).unwrap();
        self.images.insert(index, image.clone());

        let mut image_chunks = get_chunks_from_image(&mut image);

        for mut item in image_chunks.iter_mut() {
            item.texture_index = index
        }

        println!("{:?}", &image_chunks);

        self.chunks.extend(image_chunks);
        self.image_loading_order.push(index);

        if self.images.len() > 3 {
            let next_to_remove = self.image_loading_order.remove(0);
            println!("Removing key {}", next_to_remove);

            self.images.remove(&next_to_remove);
            println!("Now hash's len is {}", self.images.len())
        }

        self.images.get(&index)
    }

    fn get_metadata(&self, _path: &str) -> Option<crate::structs::ComicMetadata> {
        None
    }
}

impl Default for DirChunkProvider {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            images: HashMap::new(),
            chunks: Vec::new(),
            last_loaded_image: -1,
            image_loading_order: Vec::new(),
        }
    }
}
