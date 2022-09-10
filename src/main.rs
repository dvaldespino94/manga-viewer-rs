use raylib::prelude::*;

use crate::archive::Archive;
use crate::archive::ArEntryInfo;
use crate::structs::{Chunk, ChunkStatus};
use crate::traits::IChunkProvider;

mod archive;
mod chunkprovider;
mod structs;
mod traits;
mod unarr;

const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

#[allow(unused)]
fn draw_loading_message(context: &mut RaylibDrawHandle<'_>, screen_rect: &Rectangle) {
    const MESSAGE_WIDTH: i32 = 126;
    const FONT_SIZE: i32 = 25;
    context.draw_text(
        "LOADING",
        (screen_rect.x as i32) + ((screen_rect.width as i32) - MESSAGE_WIDTH) / 2,
        (screen_rect.y as i32) + (screen_rect.height as i32 - FONT_SIZE) / 2,
        FONT_SIZE,
        Color::BLACK,
    );
}

#[allow(unused)]
fn draw_no_chunk_message(context: &mut RaylibDrawHandle<'_>, screen_rect: &Rectangle) {
    const MESSAGE_WIDTH: i32 = 126;
    const FONT_SIZE: i32 = 25;
    context.draw_text(
        "NO CHUNK",
        (screen_rect.x as i32) + ((screen_rect.width as i32) - MESSAGE_WIDTH) / 2,
        (screen_rect.y as i32) + (screen_rect.height as i32 - FONT_SIZE) / 2,
        FONT_SIZE,
        Color::RED,
    );
}

#[allow(unused)]
fn draw_current_chunk(context: &mut RaylibDrawHandle<'_>, screen_rect: &Rectangle) {
    context.draw_rectangle(
        screen_rect.x as i32,
        screen_rect.y as i32,
        screen_rect.width as i32,
        screen_rect.height as i32,
        Color::WHITE,
    );
}

//Get chunk metadata from image
fn get_chunks_from_image(image: &mut Image) -> Vec<Chunk> {
    //How many white strips counts as a chunk separator
    const WHITE_STRIP_THRESHOLD: usize = 5;

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
                    status: ChunkStatus::Idle,
                    rect: Rectangle::new(0.0, last_chunk_start as f32, image.width as f32, height as f32),
                });
                last_chunk_start = y+1;
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
    //If last strip was true then add the last part as a chunk
    if *white_strip_map.last().unwrap() {
        chunks.push(Chunk {
            status: ChunkStatus::Idle,
            rect: Rectangle::new(
                0.0,
                last_chunk_start as f32,
                image.width as f32,
                (image.height - last_chunk_start as i32) as f32,
            ),
        })
    }

    // for y in 0..white_strip_map.len() {
    //     let last_line_was_blank = last_change.0;
    //     let its_the_first_line = y == 0;

    //     if white_strip_map[y] != last_change.0 && (y - last_change.1 > WHITE_STRIP_THRESHOLD) {
    //         last_change = (white_strip_map[y], y);
    //         chunk_meta.push(last_change.clone());

    //         if white_strip_map[y] && (last_line_was_blank || its_the_first_line) {
    //             let height = y - last_change.1;
    //             chunks.push(Chunk {
    //                 status: ChunkStatus::Idle,
    //                 rect: Rectangle::new(0.0, y as f32, image.width as f32, height as f32),
    //             });
    //         }
    //     }
    // }

    const MIN_CHUNK_HEIGHT: f32 =20.0;
    return chunks.into_iter().filter(|x| x.rect.height>MIN_CHUNK_HEIGHT).collect();
}

fn process_page<'a>(
    _context: &mut RaylibDrawHandle<'_>,
    archive: Archive,
    entry: &ArEntryInfo,
) -> (Option<Image>, Vec<Chunk>) {
    let data = archive
        .read(entry.offset, entry.size)
        .expect("Error getting image data");

    //TODO: Get extension from file name
    match Image::load_image_from_mem(".jpg", &data, data.len().try_into().unwrap()) {
        Ok(mut image) => {
            let chunks = get_chunks_from_image(&mut image);
            return (Some(image), chunks);
        }
        Err(_) => {
            println!("Error loading image from {}", entry.name);
        }
    };

    (None, Vec::new())
}

fn main() {
    let mut archive: Archive = Archive::new(
        &"/Users/david.valdespino/Downloads/Spy x Family/Spy x Family - Tomo 02 (#006-011).cbr"
            .to_string(),
    );

    let (mut rl, thread) = init()
        //Set Window Size
        .size(400, 200)
        .resizable()
        //Set Window Title
        .title("Manga Viewer")
        //Finally call build to get the context initialized
        .build();

    let _current_chunk: Option<Chunk> = None;
    let _current_chunk_index = 0;

    let app_version_string = format!(
        "{:}.{:02}.{:02}",
        APP_VERSION.0, APP_VERSION.1, APP_VERSION.2
    );

    let mut last_chunks: Vec<Chunk> = Vec::new();
    let mut last_image: Option<Image> = None;
    let mut last_image_size: Vector2 = Vector2::new(0.0, 0.0);

    let mut chunk_index = 0;
    while !rl.window_should_close() {
        if last_image.is_some() {
            last_image_size = Vector2::new(
                last_image.as_ref().unwrap().width as f32,
                last_image.as_ref().unwrap().height as f32,
            );
        }

        let mut context = rl.begin_drawing(&thread);

        let screen_rect = Rectangle::new(
            0.0,
            0.0,
            context.get_screen_width() as f32,
            context.get_screen_height() as f32,
        );

        context.clear_background(Color::LIGHTGRAY);

        //Draw the header and version
        context.draw_text(APP_TITLE, 5, 5, 12, Color::BLACK);
        context.draw_text(&app_version_string, 60, 20, 10, Color::DARKGRAY);
        {
            let scale_coeff: f32 = last_image_size.y / (screen_rect.height as f32);
            let x_offset = (screen_rect.width - (last_image_size.x / scale_coeff)) / 2.0;
            let y_offset = (screen_rect.height - (last_image_size.y / scale_coeff)) / 2.0;

            context.draw_rectangle(
                ((screen_rect.width - (last_image_size.x / scale_coeff)) / 2.0) as i32,
                ((screen_rect.height - (last_image_size.y / scale_coeff)) / 2.0) as i32,
                (last_image_size.x / scale_coeff) as i32,
                (last_image_size.y / scale_coeff) as i32,
                Color::BLACK,
            );

            for chunk in last_chunks {
                let x = x_offset + (chunk.rect.x) / scale_coeff;
                let y = y_offset + (chunk.rect.y) / scale_coeff;
                let w = (chunk.rect.width) / scale_coeff;
                let h = (chunk.rect.height) / scale_coeff;

                context.draw_rectangle_lines(x as i32, y as i32, w as i32, h as i32, Color::GRAY);
            }
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }

        match archive.next() {
            Some(page) => {
                (last_image, last_chunks) = process_page(&mut context, archive, &page);

                let mut img = last_image.clone().unwrap();
                img.set_format(raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);

                for chunk in last_chunks.iter() {
                    // let mut rect=chunk.rect;
                    // rect.x+=10.0;
                    // rect.width-=20.0;

                    // img.draw_rectangle_lines(rect, 2, Color::RED);
                    let new_image: Image = last_image.as_ref().unwrap().from_image(chunk.rect);
                    new_image.export_image(format!("./out/chunk_{chunk_index:02}.jpg").as_str());
                    chunk_index += 1;
                }
                // img.export_image(format!("chunk_{chunk_index:02}.jpg").as_str());

                println!("Got {:?} chunks", last_chunks.len());
                println!("Page: {:?}", page);
            }
            None => return,
        };
    }
}
