use std::path::Path;

use crate::{
    archive::{ArEntryInfo, Archive},
    structs::Chunk,
};
use raylib::math::Rectangle;
use raylib::prelude::Image;

//Get chunk metadata from image
#[allow(unused)]
pub fn get_chunks_from_image(image: &mut Image) -> Vec<Chunk> {
    //How many white strips counts as a chunk separator
    const WHITE_STRIP_THRESHOLD: usize = 5;

    //Minimal height a chunk must have to be recognized as such
    const MIN_CHUNK_HEIGHT: usize = WHITE_STRIP_THRESHOLD + 1;

    //Set image format to 8bit grayscale, to decrease processing costs
    image.set_format(raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAYSCALE);

    //Get image's color data (w*h*depth bytes)
    let colors = image.get_image_data();

    //Bitmap of horizontal white strips
    let mut white_strip_map = Vec::<bool>::new();

    //Iterate vertically over the whole image
    for y in 0..image.height {
        //The strips are marked as white by default
        let mut white = true;

        //Iterate over the whole strip until non-white pixels are found
        for x in 0..image.width {
            //Transform x,y coordinates to linear pixel offset
            let offset: usize = (x + y * image.width).try_into().unwrap();

            //Compare the color and a white threshold
            if colors[offset].r < 250 {
                white = false;
                break;
            }
        }

        //Push the not-white status of the line
        white_strip_map.push(white);
    }

    //Push an extra white line, so the last chunk is always added to resulting Vec
    white_strip_map.push(true);

    //Chunk vector
    let mut chunks = Vec::<Chunk>::new();

    //Initialize state-machine
    let mut last = white_strip_map[0];
    let mut last_chunk_start = 0;

    //Starts in 1 cause first item was taken as initial status anyways
    let mut y = 1;

    while y < white_strip_map.len() {
        //Act only at borders
        if last != white_strip_map[y] {
            //If a chunk end was found
            if white_strip_map[y] {
                //The chunk's height is calculated
                let height = y - last_chunk_start;
                chunks.push(Chunk {
                    rect: Rectangle::new(
                        0.0,
                        last_chunk_start as f32,
                        image.width as f32,
                        height as f32,
                    ),
                    texture_index: 0,
                });
                last_chunk_start = y + 1;
            } else {
                //Register that a chunk started here
                last_chunk_start = y;
            }
        }

        //Update state-machine
        last = white_strip_map[y];

        //Increment y
        y += 1;
    }

    return chunks
        .into_iter()
        .filter(|x| x.rect.height > (MIN_CHUNK_HEIGHT as f32))
        .collect();
}

#[allow(unused)]
pub fn process_page<'a>(archive: Archive, entry: &ArEntryInfo) -> Vec<Chunk> {
    let data = archive
        .read(entry.offset, entry.size)
        .expect("Error getting image data");

    let path = Path::new(&entry.name);
    let extension = path.extension().unwrap().to_str().unwrap();

    match Image::load_image_from_mem(
        format!(".{extension}").as_str(),
        &data,
        data.len().try_into().unwrap(),
    ) {
        Ok(mut image) => {
            let chunks = get_chunks_from_image(&mut image);
            return chunks;
        }
        Err(_) => {
            eprintln!("Error loading image from {}", entry.name);
        }
    };

    Vec::new()
}

// #[allow(unused)]
// fn extract_metadata(archive: &mut Archive) {
//     let mut page_index = 0;
//     let mut chunk_index = 0;

//     let mut db = Connection::open("data.sqlite").expect("Error opening connection with database");
//     db.execute(
//         "CREATE TABLE IF NOT EXISTS Chunks(\
//         id INTEGER PRIMARY KEY,\
//         page INTEGER,\
//         x INTEGER,\
//         y INTEGER,\
//         w INTEGER,\
//         h INTEGER\
//     )",
//         [],
//     )
//     .expect("Error creating table");

//     match archive.next() {
//         Some(page) => {
//             let chunks = process_page(*archive, &page);

//             for chunk in chunks.iter() {
//                 let tx = db.transaction().expect("Error starting transaction!");

//                 tx.execute(
//                     "INSERT INTO Chunks VALUES(?, ?, ?, ?, ?, ?);",
//                     [
//                         chunk_index,
//                         page_index,
//                         chunk.rect.x as i32,
//                         chunk.rect.y as i32,
//                         chunk.rect.width as i32,
//                         chunk.rect.height as i32,
//                     ],
//                 )
//                 .expect("Error inserting Chunk into DB");

//                 tx.commit().expect("Error commiting transaction");

//                 chunk_index += 1;
//             }

//             page_index += 1;
//         }
//         None => return,
//     };
// }
