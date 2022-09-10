use std::sync::mpsc::{Receiver, Sender};

use raylib::prelude::*;

use crate::structs::Message;
use crate::archive::{Archive, ArEntryInfo};
use crate::structs::{Chunk};
use crate::traits::{IChunkProvider};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ChunkProvider<'a> {
    pub archive: Archive,
    pub entries: Vec<ArEntryInfo<'a>>,
    chunks: Vec<Chunk>,
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl<'a> ChunkProvider<'a> {
    #[allow(unused)]
    pub fn new(_path: String) -> Self {
        todo!()
        // let archive = Archive::new(&path);

        // let (mtx, mrx): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();
        // let (_stx, srx): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();

        // std::thread::spawn(move || {
        //     let (_rl, _context) = init().build();
        //     loop {
        //         let mut chunk_query_count = 0;

        //         match srx.recv() {
        //             Ok(msg) => match msg {
        //                 Message::Command(cmd) => match cmd {
        //                     ViewerCommand::MoreChunks { how_many } => chunk_query_count = how_many,
        //                 },
        //                 Message::ChunkData(_) => {}
        //             },
        //             Err(_) => {
        //                 return;
        //             }
        //         }

        //         while chunk_query_count > 0 {}
        //     }
        // });

        // return ChunkProvider {
        //     archive,
        //     entries: archive.collect(),
        //     chunks: Vec::new(),
        //     tx: mtx,
        //     rx: mrx,
        // };
    }
}

impl IChunkProvider for ChunkProvider<'_> {
    fn get_chunk(&self, _: usize) -> Option<Chunk> {
        None
    }

    fn chunk_count(&self) -> usize {
        return self.chunks.len();
    }

    fn done_processing(&self) -> bool {
        todo!()
    }

    fn destroy(&self) {
        todo!("Implement this!");
    }

    fn open(_path: &str) -> Self {
        todo!()
    }

    fn get_texture(&self, index: usize) -> Option<Texture2D>{
        None
    }
}


pub struct DummyChunkProvider{
    _chunk_count: usize,
    _done: bool,
}

impl IChunkProvider for DummyChunkProvider {
    fn get_chunk(&self, index: usize) -> Option<Chunk> {
        None
    }

    fn chunk_count(&self) -> usize {
        self._chunk_count
    }

    fn done_processing(&self) -> bool {
        return self._done;
    }

    fn destroy(&self) {}

    fn open(path: &str) -> Self {
        return Self { _chunk_count: 0, _done: false }
    }

    fn get_texture(&self, index: usize) -> Option<Texture2D> {
        None
    }
}
