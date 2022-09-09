use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{Archive, Chunk, IChunkProvider};
use crate::archive::ArEntryInfo;
use crate::structs::Message;

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

        let (mst, msr) = channel();
        let (_smt, _smr): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();

        std::thread::spawn(move || {});

        return ChunkProvider {
            archive,
            entries: archive.collect(),
            chunks: Vec::new(),
            tx: mst,
            rx: msr,
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
