use raylib::prelude::*;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ChunkStatus {
    Idle,
    Loading,
    Ready,
}

#[derive(Debug)]
pub struct Chunk {
    pub rect: Rectangle,
    pub texture_index: usize,
    pub status: ChunkStatus,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ViewerCommand {
    MoreChunks { how_many: u32 },
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message {
    Command(ViewerCommand),
    ChunkData(Chunk),
}

//Store metadata for books, folders, etc...
#[derive(Debug)]
pub struct ComicMetadata {
    //The comic's title
    pub title: String,
    //How many chunks were found last time(Might increase)
    pub chunk_count: usize,
    //The last chunk the user was watching when application closed(Should be updated often)
    pub last_seen_chunk: usize,
}
