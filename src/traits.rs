use crate::structs::Chunk;

pub trait IChunkProvider {
    fn get_chunk(&self, index: usize) -> Option<Chunk>;
    fn chunk_count(&self) -> usize;

    fn destroy(&self);
    fn open(path: &str) -> Self;
}
