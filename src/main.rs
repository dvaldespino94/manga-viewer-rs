#![windows_subsystem = "windows"]

use std::{borrow::BorrowMut, fs::File};

use application::Application;
use log::*;
use raylib::prelude::*;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};

pub mod application;
pub mod archive;
pub mod chunkprovider;
pub mod database;
pub mod processing;
pub mod structs;
pub mod traits;
pub mod unarr;

//Constants and info for the whole application
const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

fn load_font(rl: &mut RaylibHandle, thread: &RaylibThread, size: i32) -> Result<Font, String> {
    rl.load_font_ex(
        thread,
        "./fonts/Roboto-Light.ttf",
        size,
        FontLoadEx::Default(255),
    )
}

fn main() {
    std::fs::write("/tmp/log", "Starting").unwrap();

    match simplelog::CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("manga-viewer.log").unwrap(),
        ),
    ]) {
        Ok(_) => {
            std::fs::write("/tmp/log.txt", format!("Logs created!")).unwrap();
        }
        Err(error) => {
            std::fs::write("/tmp/log.txt", format!("Error creating logs: {error}!")).unwrap();
        }
    }

    debug!("Running");

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

    debug!("Loading logo data");

    let logo_data = include_bytes!("../images/icon.png");
    let mut data = Vec::new();
    data.extend_from_slice(logo_data);

    let (logo_image, logo_texture) =
        match Image::load_image_from_mem(".png", &data, logo_data.len() as i32) {
            Ok(mut it) => {
                it.resize(50, 50);
                let texture = rl
                    .load_texture_from_image(&thread, &it)
                    .unwrap_or_else(|err| {
                        error!("Error loading texture: '{err}'");
                        rl.load_texture_from_image(
                            &thread,
                            &Image::gen_image_checked(20, 20, 4, 4, Color::BLACK, Color::WHITE),
                        )
                        .unwrap()
                    });
                (it, texture)
            }
            Err(error) => {
                error!("Error loading logo_image: '{error}'");
                return;
            }
        };

    debug!("Setting window icon");
    rl.set_window_icon(logo_image);

    //Disable closing on Escape key
    rl.set_exit_key(None);

    //Set target FPS for the application to 30
    rl.set_target_fps(30);

    debug!("Loading gui_font");
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

    debug!("Creating application");
    //Instantiate the application
    let mut app: Application = Application::new(&mut rl, &thread, logo_texture);

    //Padding for the main UI
    const PADDING: f32 = 10.0;

    debug!("Loading fonts");
    let title_font = match load_font(&mut rl, &thread, 15) {
        Ok(it) => it,
        Err(error) => {
            error!("Error loading font: '{error}'");
            return;
        }
    };

    let subtitle_font = match load_font(&mut rl, &thread, 20) {
        Ok(it) => it,
        Err(error) => {
            error!("Error loading font: '{error}'");
            return;
        }
    };

    #[cfg(target_os = "windows")]
    const MOD_KEY: KeyboardKey = KeyboardKey::KEY_LEFT_CONTROL;

    #[cfg(target_os = "macos")]
    const MOD_KEY: KeyboardKey = KeyboardKey::KEY_LEFT_SUPER;

    debug!("Starting main loop");
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

        //Get the RL Context
        let mut context = rl.begin_drawing(&thread);
        context.gui_set_font(&gui_font);

        if context.is_key_pressed(KeyboardKey::KEY_O) && context.is_key_down(MOD_KEY) {
            let fd = rfd::FileDialog::new();
            if let Some(result) = fd.pick_folder() {
                if let Err(open_result) = app.open_document(&String::from(result.to_str().unwrap()))
                {
                    error!("Error opening document: {}", open_result)
                }
            }
        }

        if context.is_key_pressed(KeyboardKey::KEY_W) && context.is_key_down(MOD_KEY) {
            app.close_document();
        }

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

        //Draw Application Logo
        context.draw_texture(&app.logo_texture, 5, 5, Color::WHITE);

        //Draw the header and version
        context.draw_text_ex(
            &subtitle_font,
            APP_TITLE,
            Vector2::new(55.0, 15.0),
            (&subtitle_font).baseSize as f32,
            0.0,
            Color::BLACK,
        );

        context.draw_text_ex(
            &title_font,
            &app_version_string,
            Vector2::new(120.0, 30.0),
            (&title_font).baseSize as f32,
            0.0,
            Color::DARKGRAY,
        );
    }

    //Unload current provider, so metadata gets saved on app quit
    app.provider.unload();
}
