use raylib::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::archive::ArEntryInfo;
use crate::structs::{Message, ViewerCommand};
use crate::{Archive, Chunk, IChunkProvider};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ChunkProvider<'a> {
    pub archive: Archive,
    pub entries: Vec<ArEntryInfo<'a>>,
    chunks: Vec<Chunk<'a>>,
    tx: Sender<Message<'a>>,
    rx: Receiver<Message<'a>>,
}

impl<'a> ChunkProvider<'a> {
    pub fn new(path: String) -> Self {
        let archive = Archive::new(&path);

        let (mtx, mrx): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();
        let (_stx, srx): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();

        std::thread::spawn(move || {
            let (_rl, _context) = init().build();
            loop {
                let mut chunk_query_count = 0;

                match srx.recv() {
                    Ok(msg) => match msg {
                        Message::Command(cmd) => match cmd {
                            ViewerCommand::MoreChunks { how_many } => chunk_query_count = how_many,
                        },
                        Message::ChunkData(_) => {}
                    },
                    Err(_) => {
                        return;
                    }
                }

                while chunk_query_count > 0 {}
            }
        });

        return ChunkProvider {
            archive,
            entries: archive.collect(),
            chunks: Vec::new(),
            tx: mtx,
            rx: mrx,
        };
    }
}

impl IChunkProvider for ChunkProvider<'_> {
    fn get_chunk(&self, _: usize) -> Option<Chunk<'_>> {
        return None;
    }

    fn chunk_count(&self) -> usize {
        return self.chunks.len();
    }

    fn destroy(&self) {
        todo!("Implement this!");
    }

    fn open(_path: &str) -> Self {
        todo!()
    }
}
