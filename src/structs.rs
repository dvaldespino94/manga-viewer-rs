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
