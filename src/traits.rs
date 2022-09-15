use raylib::texture::Image;

use crate::structs::{Chunk, ComicMetadata};

pub trait IChunkProvider {
    fn get_chunk(&mut self, index: usize) -> Option<&Chunk>;
    fn chunk_count(&self) -> usize;
    fn done_processing(&self) -> bool;

    fn destroy(&self);
    fn unload(&mut self);
    fn open(&mut self, path: &str) -> Result<ComicMetadata, String>;
    fn get_image(&mut self, index: usize) -> Option<&Image>;

    fn get_metadata(&self, path: &str) -> Result<ComicMetadata, String>;
}
