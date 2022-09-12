use raylib::prelude::*;

use crate::structs::{Chunk, ComicMetadata};
use crate::traits::IChunkProvider;

// #[derive(Debug)]
// #[allow(dead_code)]
// pub struct ChunkProvider<'a> {
//     pub archive: Archive,
//     pub entries: Vec<ArEntryInfo<'a>>,
//     chunks: Vec<Chunk>,
//     tx: Sender<Message>,
//     rx: Receiver<Message>,
// }

// impl<'a> ChunkProvider<'a> {
//     #[allow(unused)]
//     pub fn new(_path: String) -> Self {
//         todo!()
//         // let archive = Archive::new(&path);

//         // let (mtx, mrx): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();
//         // let (_stx, srx): (Sender<Message<'a>>, Receiver<Message<'a>>) = channel();

//         // std::thread::spawn(move || {
//         //     let (_rl, _context) = init().build();
//         //     loop {
//         //         let mut chunk_query_count = 0;

//         //         match srx.recv() {
//         //             Ok(msg) => match msg {
//         //                 Message::Command(cmd) => match cmd {
//         //                     ViewerCommand::MoreChunks { how_many } => chunk_query_count = how_many,
//         //                 },
//         //                 Message::ChunkData(_) => {}
//         //             },
//         //             Err(_) => {
//         //                 return;
//         //             }
//         //         }

//         //         while chunk_query_count > 0 {}
//         //     }
//         // });

//         // return ChunkProvider {
//         //     archive,
//         //     entries: archive.collect(),
//         //     chunks: Vec::new(),
//         //     tx: mtx,
//         //     rx: mrx,
//         // };
//     }
// }

// impl IChunkProvider for ChunkProvider<'_> {
//     fn get_chunk(&self, _: usize) -> Option<Chunk> {
//         None
//     }

//     fn chunk_count(&self) -> usize {
//         return self.chunks.len();
//     }

//     fn done_processing(&self) -> bool {
//         todo!()
//     }

//     fn destroy(&self) {
//         todo!("Implement this!");
//     }

//     fn open(_path: &str) -> Self {
//         todo!()
//     }

//     fn get_image(&self, index: usize) -> Option<Image> {
//         None
//     }

//     fn new() -> Self {
//         todo!()
//     }
// }

pub struct DummyChunkProvider {
    chunk_count: usize,
    done: bool,
    _image: Option<Image>,
    sizes: Vec<(i32, i32)>,
}

impl IChunkProvider for DummyChunkProvider {
    fn get_chunk(&self, index: usize) -> Option<Chunk> {
        Some(Chunk {
            rect: Rectangle {
                x: 5.0,
                y: 5.0,
                width: self.sizes[index].0 as f32,
                height: self.sizes[index].1 as f32,
            },

            texture_index: 0,
            status: crate::structs::ChunkStatus::Ready,
        })
    }

    fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    fn done_processing(&self) -> bool {
        return self.done;
    }

    fn destroy(&self) {}

    fn open(&mut self, _path: &str) -> bool {
        return true;
    }

    fn get_image(&self, _index: usize) -> Option<&Image> {
        Some(&self._image.as_ref().unwrap())
    }

    fn new() -> Self {
        const CHUNK_COUNT: usize = 10;
        let image = Image::gen_image_checked(800, 600, 10, 10, Color::BLACK, Color::WHITE);
        let mut sizes = Vec::new();

        for _ in 0..CHUNK_COUNT {
            let w = get_random_value(0, image.width);
            let h = get_random_value(0, image.height);
            sizes.push((w, h));
        }

        Self {
            chunk_count: CHUNK_COUNT,
            done: false,
            _image: Some(image),
            sizes: sizes,
        }
    }

    fn get_metadata(_: &str) -> Option<ComicMetadata> {
        Some(ComicMetadata {
            title: "Some Comic".to_string(),
            chunk_count: 10,
            last_seen_chunk: 2,
        })
    }
}
