use crate::processing::get_chunks_from_image;
use raylib::prelude::*;
use std::{cmp::max, collections::HashMap, path::Path};

use crate::{structs::Chunk, traits::IChunkProvider};

pub struct DirChunkProvider {
    document_path: String,
    files: Vec<String>,
    chunk_index: HashMap<usize, Vec<usize>>,
    chunks: Vec<Chunk>,
    images: HashMap<usize, Image>,
    image_loading_order: Vec<usize>,
    last_queried_chunk: usize,
}

impl DirChunkProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IChunkProvider for DirChunkProvider {
    fn get_chunk(&mut self, index: usize) -> Option<&crate::structs::Chunk> {
        self.last_queried_chunk = index;
        if index >= self.chunks.len() {
            eprintln!("Queried chunk #{index} wich is out of bounds");
            if self.chunk_index.len() == 0 {
                self.get_image(0);
            }
            self.get_image(*self.chunk_index.keys().max().or(Some(&0)).unwrap() + 1 as usize);
        }

        self.chunks.get(index)
    }

    fn chunk_count(&self) -> usize {
        if self.done_processing() {
            return self.chunks.len();
        }

        if self.files.len() > 0 {
            return max(self.chunks.len() + 1, 1);
        }

        return 0;
    }

    fn done_processing(&self) -> bool {
        self.chunks.len() == self.files.len()
    }

    fn destroy(&self) {
        todo!()
    }

    fn open(
        self: &mut DirChunkProvider,
        _path: &str,
        cached_chunks: Option<Vec<Chunk>>,
    ) -> Result<(), String> {
        let path = Path::new(_path);
        if path.exists() && path.is_dir() {
            if let Ok(dir) = path.read_dir() {
                self.files = dir
                    .map(|element| element.unwrap().path().to_str().unwrap().to_string())
                    .filter(|element| element.ends_with(".jpg") || element.ends_with(".png"))
                    .collect();
            }

            if let Some(chunks) = &cached_chunks {
                self.chunks = chunks.clone();
                let mut index: HashMap<usize, Vec<usize>> = HashMap::new();

                for i in 0..self.chunks.len() {
                    let c = self.chunks.get(i).unwrap();
                    let t_index = c.texture_index;
                    if !index.contains_key(&t_index) {
                        index.insert(t_index, Vec::new());
                    }

                    index.get_mut(&t_index).unwrap().push(i);
                }

                self.chunk_index = index;
            }

            //Preload first image
            self.get_image(0);

            self.document_path = _path.to_string();

            return Ok(());
        }

        return Err("Error opening document".to_string());
    }

    fn get_image(&mut self, index: usize) -> Option<&raylib::texture::Image> {
        eprintln!("Getting image {}", index);

        if index >= self.files.len() {
            return None;
        }

        if self.images.contains_key(&index) {
            return self.images.get(&index);
        }

        eprintln!("Image {} not found, fetching...", index);

        let mut image = Image::load_image(self.files[index].as_str()).unwrap();
        self.images.insert(index, image.clone());

        if index == self.chunk_index.len() {
            let mut image_chunks = get_chunks_from_image(&mut image);

            for mut item in image_chunks.iter_mut() {
                item.texture_index = index
            }

            let mut index_vec = Vec::new();
            let start_index = self.chunks.len();
            for i in 0..image_chunks.len() {
                index_vec.push(start_index + i);
                self.chunks.push(image_chunks.remove(0));
            }

            self.chunk_index.insert(index, index_vec);
        }

        self.image_loading_order.push(index);

        if self.images.len() > 3 {
            let next_to_remove = self.image_loading_order.remove(0);
            eprintln!("Removing key {}", next_to_remove);

            self.images.remove(&next_to_remove);
            eprintln!("Now hash's len is {}", self.images.len())
        }

        self.images.get(&index)
    }

    fn unload(&mut self) {
        if self.document_path.is_empty() || !Path::new(self.document_path.as_str()).exists() {
            eprintln!("Path is empty!");
            return;
        }

        self.files.clear();
        self.image_loading_order.clear();
        self.images.clear();
        self.chunks.clear();
        self.chunk_index.clear();
        self.document_path = String::new();
    }

    fn can_open(&self, document_path: &str) -> bool {
        let path = Path::new(document_path);
        return path.exists() && path.is_dir();
    }
}

impl Default for DirChunkProvider {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            images: HashMap::new(),
            chunks: Vec::new(),
            image_loading_order: Vec::new(),
            chunk_index: HashMap::new(),
            document_path: String::new(),
            last_queried_chunk: 0,
        }
    }
}
