#![windows_subsystem = "windows"]

use std::borrow::BorrowMut;

use application::Application;
use raylib::prelude::*;

pub mod application;
pub mod archive;
pub mod chunkprovider;
pub mod dirchunkprovider;
pub mod metaprovider;
pub mod processing;
pub mod structs;
pub mod traits;
pub mod unarr;

//Constants and info for the whole application
const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

fn load_font(rl: &mut RaylibHandle, thread: &RaylibThread, size: i32) -> Font {
    let font: Font = rl
        .load_font_ex(
            thread,
            "./fonts/Roboto-Light.ttf",
            size,
            FontLoadEx::Default(255),
        )
        .expect("Can't load default font!");

    font
}

fn main() {
    //Initialze RayGUI
    let (mut rl, thread) = init()
        //Set Window Size
        .size(700, 400)
        //Make window resizable
        .resizable()
        //Set Window Title
        .title("Manga Viewer")
        //Finally call build to get the context initialized
        .build();

    //Disable closing on Escape key
    rl.set_exit_key(None);

    //Set target FPS for the application to 30
    rl.set_target_fps(30);

    let gui_font = rl
        .load_font_ex(
            &thread,
            "./fonts/Roboto-Light.ttf",
            12,
            FontLoadEx::Default(255),
        )
        .unwrap();

    //Format the Version string
    let app_version_string = format!("{:}.{:2}.{:2}", APP_VERSION.0, APP_VERSION.1, APP_VERSION.2);

    //Instantiate the application
    let mut app: Application = Application::new(&mut rl, &thread);

    //Padding for the main UI
    const PADDING: f32 = 10.0;

    let title_font = load_font(&mut rl, &thread, 15);
    let subtitle_font = load_font(&mut rl, &thread, 20);

    //RayLib's mainloop
    while !rl.borrow_mut().window_should_close() {
        app.load_textures(rl.borrow_mut(), &thread);

        //Clear query vec
        app.image_queries.clear();

        if rl.is_file_dropped() {
            if let Ok(_) = app.open_document(&rl.get_dropped_files()[0]) {
            } else {
            }
            rl.clear_dropped_files();
        }

        {
            //Get the RL Context
            let mut context = rl.begin_drawing(&thread);
            context.gui_set_font(&gui_font);

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
            context.draw_text_ex(
                &subtitle_font,
                APP_TITLE,
                Vector2::new(5.0, 5.0),
                (&subtitle_font).baseSize as f32,
                0.0,
                Color::BLACK,
            );

            context.draw_text_ex(
                &title_font,
                &app_version_string,
                Vector2::new(60.0, 20.0),
                (&title_font).baseSize as f32,
                0.0,
                Color::DARKGRAY,
            );
        }
    }

    //Unload current provider, so metadata gets saved on app quit
    app.provider.unload();
}
