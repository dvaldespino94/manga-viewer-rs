use raylib::texture::Image;

use crate::structs::{Chunk, ComicMetadata};

pub trait IChunkProvider {
    fn new() -> Self;

    fn get_chunk(&mut self, index: usize) -> Option<&Chunk>;
    fn chunk_count(&self) -> usize;
    fn done_processing(&self) -> bool;

    fn destroy(&self);
    fn open(&mut self, path: &str) -> bool;
    fn get_image(&mut self, index: usize) -> Option<&Image>;

    fn get_metadata(path: &str) -> Option<ComicMetadata>;
}
