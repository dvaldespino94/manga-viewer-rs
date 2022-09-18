use raylib::prelude::Image;

use crate::{traits::IChunkProvider, structs::Chunk};

use super::dirchunkprovider::DirChunkProvider;


pub struct MetaProvider {
    providers: Vec<Box<dyn IChunkProvider>>,
    current_provider_index: usize,
}

impl MetaProvider {
    pub fn new() -> Self {
        let _self = Self {
            current_provider_index: 0,
            providers: Vec::from([Box::new(DirChunkProvider::new()) as Box<dyn IChunkProvider>]),
        };

        _self
    }

    pub fn current_provider(&self) -> &Box<dyn IChunkProvider> {
        &self.providers[self.current_provider_index]
    }

    pub fn current_provider_mut(&mut self) -> &mut Box<dyn IChunkProvider> {
        &mut self.providers[self.current_provider_index]
    }
}

impl IChunkProvider for MetaProvider {
    fn get_chunk(&mut self, index: usize) -> Option<&Chunk> {
        self.current_provider_mut().get_chunk(index)
    }

    fn chunk_count(&self) -> usize {
        self.current_provider().chunk_count()
    }

    fn done_processing(&self) -> bool {
        self.current_provider().done_processing()
    }

    fn destroy(&self) {
        self.current_provider().destroy()
    }

    fn open(&mut self, path: &str) -> Result<(), String> {
        let mut index = 0;

        //Get a provider that can handle this file format
        for provider in self.providers.iter() {
            if provider.can_open(path) {
                self.current_provider_index = index;
                break;
            }
            index += 1;
        }

        //If there was no situable provider just return false
        if index >= self.providers.len() {
            return Err("No situable provider found!".to_string());
        }

        self.current_provider_mut().open(path)
    }

    fn get_image(&mut self, index: usize) -> Option<&Image> {
        self.current_provider_mut().get_image(index)
    }

    fn unload(&mut self) {
        self.current_provider_mut().unload();
    }

    fn can_open(&self, path: &str) -> bool {
        for provider in self.providers.iter() {
            if provider.can_open(path) {
                return true;
            }
        }
        false
    }
}
