use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc::channel;

use crate::Archive;
use crate::archive::ArEntryInfo;
use crate::traits::IChunkProvider;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ChunkStatus {
    Loading,
    Ready,
}

#[derive(Debug)]
pub struct Chunk<'a> {
    pub index: u16,
    pub path: &'a str,
    pub status: ChunkStatus,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ViewerCommand {
    MoreChunks { how_many: u32 },
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message<'a> {
    Command(ViewerCommand),
    ChunkData(Chunk<'a>),
}
