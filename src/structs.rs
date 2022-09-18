use raylib::prelude::*;

#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone)]
pub struct ComicMetadata {
    //The comic's title
    pub title: String,
    //How many chunks were found last time(Might increase)
    pub chunk_count: usize,
    //The last chunk the user was watching when application closed(Should be updated often)
    pub last_seen_chunk: usize,
    //Document Path
    pub path: String,
    //Thumbnail
    pub thumbnail: Option<String>,
}

impl Default for ComicMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            chunk_count: 0,
            last_seen_chunk: 0,
            path: String::from(""),
            thumbnail: None,
        }
    }
}
