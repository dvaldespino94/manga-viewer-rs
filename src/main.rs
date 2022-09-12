use std::borrow::BorrowMut;

use application::Application;
use raylib::prelude::*;

pub mod processing;
pub mod application;
pub mod archive;
pub mod chunkprovider;
pub mod dirchunkprovider;
pub mod structs;
pub mod traits;
pub mod unarr;

//Constants and info for the whole application
const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

fn main() {
    //Initialze RayGUI
    let (mut rl, thread) = init()
        //Set Window Size
        .size(400, 200)
        //Make window resizable
        .resizable()
        //Set Window Title
        .title("Manga Viewer")
        //Finally call build to get the context initialized
        .build();

    //Set target FPS for the application to 30
    rl.set_target_fps(30);

    //Format the Version string
    let app_version_string = format!(
        "{:}.{:02}.{:02}",
        APP_VERSION.0, APP_VERSION.1, APP_VERSION.2
    );

    //Instantiate the application
    let mut app: Application = Application::new();

    //Padding for the main UI
    const PADDING: f32 = 10.0;

    //RayLib's mainloop
    while !rl.borrow_mut().window_should_close() {
        app.load_textures(rl.borrow_mut(), &thread);

        //Clear query vec
        app.image_queries.clear();

        if rl.is_file_dropped() {
            app.handle_dropped_document(rl.get_dropped_files()[0].to_string());
            rl.clear_dropped_files();
        }

        {
            //Get the RL Context
            let mut context = rl.begin_drawing(&thread);

            //Cache screen rectangle, adding offsets and correcting width/height
            let screen_rect = Rectangle::new(
                PADDING,
                PADDING + 30f32,
                context.get_screen_width() as f32 - 2f32 * PADDING,
                context.get_screen_height() as f32 - 2f32 * PADDING - 30f32,
            );

            //Clear the screen's background
            context.clear_background(Color::WHITE);

            //Draw the application
            app.draw(screen_rect, &mut context);

            //Draw the header and version
            context.draw_text(APP_TITLE, 5, 5, 12, Color::BLACK);
            context.draw_text(&app_version_string, 60, 20, 10, Color::DARKGRAY);
        }
    }
}
