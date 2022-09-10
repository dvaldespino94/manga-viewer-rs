use raylib::prelude::*;
use crate::structs::{Chunk};
use crate::chunkprovider::{ChunkProvider};
use crate::traits::{IChunkProvider};

pub mod traits;
pub mod structs;
pub mod chunkprovider;
pub mod archive;
pub mod unarr;

const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

struct Application<T: IChunkProvider> {
    current_page: usize,
    current_chunk: usize,
    provider: Option<T>,
}

impl<T: IChunkProvider> Application<T> {
    pub fn new() -> Self {
        Self {
            current_page: 0,
            current_chunk: 0,
provider:            None,
        }
    }
 
    pub(crate) fn draw(&self, screen_rect: Rectangle, context: &mut RaylibDrawHandle) {
        context.draw_rectangle_lines_ex(screen_rect, 1, Color::DARKGRAY);
    }
}

fn main() {
    //Initialze RayGUI
    let (mut rl, thread) = init()
        //Set Window Size
        .size(400, 200)
        .resizable()
        //Set Window Title
        .title("Manga Viewer")
        //Finally call build to get the context initialized
        .build();

    let app_version_string = format!(
        "{:}.{:02}.{:02}",
        APP_VERSION.0, APP_VERSION.1, APP_VERSION.2
    );

    let mut app:Application<ChunkProvider> = Application::new();

    const PADDING: f32 = 10.0;
    while !rl.window_should_close() {
        let mut context = rl.begin_drawing(&thread);

        let screen_rect = Rectangle::new(
            PADDING,
            PADDING + 30f32,
            context.get_screen_width() as f32 - 2f32 * PADDING,
            context.get_screen_height() as f32 - 2f32 * PADDING - 30f32,
        );

        context.clear_background(Color::LIGHTGRAY);

        //Draw the header and version
        context.draw_text(APP_TITLE, 5, 5, 12, Color::BLACK);
        context.draw_text(&app_version_string, 60, 20, 10, Color::DARKGRAY);

        app.draw(screen_rect, &mut context);
    }
}
