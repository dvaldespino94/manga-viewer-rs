use std::collections::HashMap;

use crate::chunkprovider::DummyChunkProvider;
use crate::traits::IChunkProvider;
use raylib::prelude::*;

pub mod archive;
pub mod chunkprovider;
pub mod structs;
pub mod traits;
pub mod unarr;

const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

struct Application<T: IChunkProvider> {
    current_page: usize,
    current_chunk: usize,
    provider: Option<T>,
    image_queries: Vec<usize>,
    textures: HashMap<usize, Option<Texture2D>>,
}

impl<'a, T: IChunkProvider> Application<T> {
    pub fn new() -> Self {
        let provider: Option<T> = Some(T::new());
        Self {
            current_page: 0,
            current_chunk: 0,
            provider,
            image_queries: Vec::new(),
            textures: HashMap::new(),
        }
    }

    #[inline]
    pub fn draw(&mut self, screen_rect: Rectangle, context: &mut RaylibDrawHandle) {
        self.handle_input(context);

        context.draw_rectangle_lines_ex(screen_rect, 1, Color::DARKGRAY);

        let provider = self.provider.as_ref().unwrap();
        let texture: &Option<Texture2D> = if self.textures.contains_key(&self.current_page) {
            // println!("Getting texture from cache!");
            self.textures.get(&self.current_page).unwrap()
        } else {
            // println!("Requesting image {}", self.current_page);
            self.image_queries.push(self.current_page);
            &None
        };

        let chunk = provider.get_chunk(self.current_chunk);

        if let Some(t) = texture {
            if let Some(c) = chunk {
                let mut target_rect = screen_rect;
                let coeff = target_rect.width / target_rect.height;

                // target_rect.height -= 50.0;
                // target_rect.y += 50.0;
                // target_rect.x+=10.0;
                // target_rect.width-=10.0;

                context.draw_texture_pro(
                    texture.as_ref().unwrap(),
                    c.rect,
                    target_rect,
                    Vector2::zero(),
                    0f32,
                    Color::WHITE,
                )
            }
        } else {
            context.draw_text(
                "No Texture",
                screen_rect.x as i32 + 10,
                screen_rect.y as i32 + 10,
                14,
                Color::BLACK,
            );
        }

        //TODO: Replace progressbar with dot/fraction representation from previous D implementations
        context.gui_progress_bar(
            Rectangle::new(
                screen_rect.x + 5.0,
                screen_rect.y + 2.0,
                screen_rect.width - 10.0,
                10.0,
            ),
            None,
            None,
            self.current_chunk as f32,
            0.0,
            (self.provider.as_ref().unwrap().chunk_count() - 1) as f32,
        );
    }

    fn handle_input(&mut self, context: &mut RaylibDrawHandle) {
        let mut something_changed = false;

        if context.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN)
            || context.is_key_pressed(KeyboardKey::KEY_RIGHT)
        {
            self.current_chunk += 1;
            something_changed = true;
        } else if context.is_key_pressed(KeyboardKey::KEY_PAGE_UP)
            || context.is_key_pressed(KeyboardKey::KEY_LEFT)
        {
            if self.current_chunk > 0 {
                self.current_chunk -= 1;
            }
            something_changed = true;
        }

        if something_changed {
            if let Some(provider) = self.provider.as_ref() {
                if self.current_chunk >= provider.chunk_count() {
                    self.current_chunk = provider.chunk_count() - 1;
                }
            }
        }
    }
}

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

    let app_version_string = format!(
        "{:}.{:02}.{:02}",
        APP_VERSION.0, APP_VERSION.1, APP_VERSION.2
    );

    let mut app: Application<DummyChunkProvider> = Application::new();

    const PADDING: f32 = 10.0;
    while !rl.window_should_close() {
        for query in app.image_queries.iter() {
            println!("Loading texture {:?}", query);

            if let Some(image) = app.provider.as_ref().unwrap().get_image(*query) {
                let value = Some(rl.load_texture_from_image(&thread, image).unwrap());
                app.textures.insert(*query, value);
            }
        }

        app.image_queries.clear();

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
