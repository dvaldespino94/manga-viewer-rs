use std::collections::HashMap;

use crate::chunkprovider::DummyChunkProvider;
use crate::traits::IChunkProvider;
use raylib::prelude::*;

pub mod archive;
pub mod chunkprovider;
pub mod structs;
pub mod traits;
pub mod unarr;

//Constants and info for the whole application
const APP_TITLE: &str = "Manga Viewer";
const APP_VERSION: (i8, i8, i8) = (0, 1, 0);

//Main Application class, holds viewer state and provider
struct Application<T: IChunkProvider> {
    //Current chunk index
    current_chunk: usize,
    //Chunk/Texture provider
    provider: Option<T>,
    //Images to be converted into textures
    image_queries: Vec<usize>,
    //Hash keeping the textures
    textures: HashMap<usize, Option<Texture2D>>,
}

//TODO: Keep textures hash as light as possible, freeing non used textures
impl<'a, T: IChunkProvider> Application<T> {
    /// Creates a new [`Application<T>`].
    pub fn new() -> Self {
        //Instantiate the provider
        let provider: Option<T> = Some(T::new());
        //Return a new application
        Self {
            current_chunk: 0,
            provider,
            image_queries: Vec::new(),
            textures: HashMap::new(),
        }
    }

    #[inline]
    //Draw Application
    pub fn draw(&mut self, screen_rect: Rectangle, context: &mut RaylibDrawHandle) {
        //Handle user input
        self.handle_input(context);

        //Draw borders(this is intended for debugging only)
        context.draw_rectangle_lines_ex(screen_rect, 1, Color::DARKGRAY);

        //Unwrap a reference to the provider
        let provider = self.provider.as_ref().unwrap();
        //If a chunk can be retrieved from the provider
        if let Some(chunk) = provider.get_chunk(self.current_chunk) {
            //Check if there is a texture already loaded from the provider
            let texture: &Option<Texture2D> = if self.textures.contains_key(&chunk.texture_index) {
                // println!("Getting texture from cache!");
                //Unwrap the texture from the local hash
                self.textures.get(&chunk.texture_index).unwrap()
            } else {
                // println!("Requesting image {}", chunk.texture_index);
                //Add the texture index to texture-query list
                self.image_queries.push(chunk.texture_index);
                &None
            };

            //If the texture exists in cache
            if let Some(t) = texture {
                //Calculate the target rectangle for the texture
                let mut target_rect = screen_rect;

                //Width/Height Ratio
                let coeff = target_rect.width / target_rect.height;

                //Draw the texture
                context.draw_texture_pro(
                    texture.as_ref().unwrap(),
                    chunk.rect,
                    target_rect,
                    Vector2::zero(),
                    0f32,
                    Color::WHITE,
                )
            } else {
                //Draw a label with a "No Texture" message
                context.draw_text(
                    "No Texture",
                    screen_rect.x as i32 + 10,
                    screen_rect.y as i32 + 10,
                    14,
                    Color::BLACK,
                );
            }
        };

        //Y position for the indicator
        let y = screen_rect.y - 2.0 + screen_rect.height;
        //Available width
        let available_w = screen_rect.width - 10.0;
        //Offset between dots
        let offset = 10.0;
        //How many dots to draw
        let chunk_count = provider.chunk_count() as f32;

        //Actual width occupied by the dots
        let w = (offset) * chunk_count;
        //Does it fit in the available space?
        let it_fits = w < available_w;

        //If it does then draw the dots, else draw a fraction
        if it_fits {
            //Starting x offset for the dots
            let x_offset = (screen_rect.x + 5.0 + (available_w - w) / 2.0) as i32;

            for i in 0..(chunk_count as i32) {
                let (r, color) = if i == (self.current_chunk as i32) {
                    (4.0, Color::BLUE)
                } else {
                    (2.0, Color::GRAY)
                };
                context.draw_circle(x_offset + i * (offset) as i32, y as i32, r, color);
            }
        } else {
            //Current chunk (Applying 1-based index offset)
            let current = self.current_chunk + 1;
            //Fraction message
            let fraction = format!("{current:03} / {chunk_count:03}");
            //Calculate message width
            let message_width = measure_text(fraction.as_str(), 14);
            //Starting x offset for fraction
            let x_offset = screen_rect.x + 5.0 + (available_w - message_width as f32) / 2.0;

            //Draw the fraction message
            context.draw_text(
                fraction.as_str(),
                x_offset as i32,
                y as i32,
                14,
                Color::BLACK,
            );
        }
    }

    //Handle user input
    fn handle_input(&mut self, context: &mut RaylibDrawHandle) {
        //Flag to signal that the user pressed next/prev or scrolled the image
        let mut something_changed = false;

        //Check for simple next/prev events
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

        //Keep current_chunk into bounds
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

    //Format the Version string
    let app_version_string = format!(
        "{:}.{:02}.{:02}",
        APP_VERSION.0, APP_VERSION.1, APP_VERSION.2
    );

    //Instantiate the application
    let mut app: Application<DummyChunkProvider> = Application::new();

    //Padding for the main UI
    const PADDING: f32 = 10.0;
    //RayLib's mainloop
    while !rl.window_should_close() {
        //Check for texture queries
        for query in app.image_queries.iter() {
            println!("Loading texture {:?}", query);

            //Try to get the image from the provider
            if let Some(image) = app.provider.as_ref().unwrap().get_image(*query) {
                //Get the texture from the image
                let value = Some(rl.load_texture_from_image(&thread, image).unwrap());
                //Insert the texture into the app's index/texture hash
                app.textures.insert(*query, value);
            }
        }

        //Clear query vec
        app.image_queries.clear();

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
        context.clear_background(Color::LIGHTGRAY);

        //Draw the application
        app.draw(screen_rect, &mut context);

        //Draw the header and version
        context.draw_text(APP_TITLE, 5, 5, 12, Color::BLACK);
        context.draw_text(&app_version_string, 60, 20, 10, Color::DARKGRAY);
    }
}
