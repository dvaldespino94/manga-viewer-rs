use raylib::prelude::*;

use crate::archive::Archive;
use crate::chunkprovider::ChunkProvider;
use crate::structs::{Chunk, ChunkStatus};
use crate::traits::IChunkProvider;

mod archive;
mod chunkprovider;
mod structs;
mod traits;
mod unarr;

const APP_TITLE: &str = "Manga Viewer";
const APP_VESION: (i8, i8, i8) = (0, 1, 0);

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

fn draw_current_chunk(context: &mut RaylibDrawHandle<'_>, screen_rect: &Rectangle) {
    context.draw_rectangle(
        screen_rect.x as i32,
        screen_rect.y as i32,
        screen_rect.width as i32,
        screen_rect.height as i32,
        Color::WHITE,
    );
}

fn main() {
    let provider: ChunkProvider = ChunkProvider::new(
        "/Users/david.valdespino/Downloads/Spy x Family/Spy x Family - Tomo 02 (#006-011).cbr"
            .to_string(),
    );

    let (mut rl, thread) = init()
        //Set Window Size
        .size(400, 200)
        //Set Window Title
        .title("Manga Viewer")
        //Finally call build to get the context initialized
        .build();

    let mut current_chunk: Option<Chunk> = None;
    let current_chunk_index = 0;
    let app_version_string = format!("{:}.{:02}.{:02}", APP_VESION.0, APP_VESION.1, APP_VESION.2);

    while !rl.window_should_close() {
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

        match current_chunk {
            Some(ref chunk) => match chunk.status {
                ChunkStatus::Loading => {
                    draw_loading_message(&mut context, &screen_rect);
                }
                ChunkStatus::Ready => {
                    draw_current_chunk(&mut context, &screen_rect);
                }
            },
            None => {
                current_chunk = provider.get_chunk(current_chunk_index);
                draw_no_chunk_message(&mut context, &screen_rect);
            }
        }
    }
}
