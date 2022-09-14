use std::borrow::BorrowMut;

use raylib::prelude::Image;

use crate::{dirchunkprovider::DirChunkProvider, structs::Chunk, traits::IChunkProvider};

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

    fn open(&mut self, path: &str) -> bool {
        self.current_provider_mut().open(path)
    }

    fn get_image(&mut self, index: usize) -> Option<&Image> {
        self.current_provider_mut().get_image(index)
    }

    fn get_metadata(&self, path: &str) -> Option<crate::structs::ComicMetadata> {
        self.current_provider().get_metadata(path)
    }
}
