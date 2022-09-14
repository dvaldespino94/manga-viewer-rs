use raylib::prelude::*;

#[derive(Debug)]
pub struct Chunk {
    pub rect: Rectangle,
    pub texture_index: usize,
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
