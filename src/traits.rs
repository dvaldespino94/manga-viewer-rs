use raylib::texture::Texture2D;

use crate::structs::Chunk;

pub trait IChunkProvider {
    fn get_chunk(&self, index: usize) -> Option<Chunk>;
    fn chunk_count(&self) -> usize;
    fn done_processing(&self) -> bool;

    fn destroy(&self);
    fn open(path: &str) -> Self;
    fn get_texture(&self, index: usize) -> Option<Texture2D>;
}
