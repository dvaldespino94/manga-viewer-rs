use std::{
    collections::HashMap,
    ffi::CString,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
};

use raylib::prelude::*;

use crate::{metaprovider::MetaProvider, structs::Chunk, traits::IChunkProvider};

//Main Application class, holds viewer state and provider
pub struct Application {
    //Current chunk index
    current_chunk_index: usize,
    //Current chunk cached instance
    current_chunk: Option<Chunk>,
    //Chunk/Texture provider
    pub provider: Box<dyn IChunkProvider>,
    //Images to be converted into textures
    pub image_queries: Vec<usize>,
    //Hash keeping the textures
    pub textures: HashMap<usize, Option<Texture2D>>,
    //Image scroll offset
    scroll: f32,
    //Smoothed scroll offset
    smoothed_scroll: f32,
    //Recent documents list
    recent_documents: Vec<String>,
    //Texture indexes in the order they were loaded
    texture_loading_order: Vec<usize>,
    //Error messages
    pub errors: Vec<(String, String, Option<fn()>)>,
}

impl<'a> Application {
    /// Creates a new [`Application`].
    pub fn new() -> Self {
        //Return a new application
        Self {
            current_chunk_index: 0,
            provider: Box::new(MetaProvider::new()),
            current_chunk: None,
            image_queries: Vec::new(),
            textures: HashMap::new(),
            scroll: 0.0,
            smoothed_scroll: 0.0,
            //Load recent files from 'recent.txt'
            recent_documents: if let Ok(text) = std::fs::read_to_string("recent.txt") {
                text.split("\n")
                    .map(|x| String::from(x))
                    .filter(|x| Path::new(x).exists())
                    .collect()
            } else {
                Vec::new()
            },
            texture_loading_order: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn load_textures(&mut self, context: &mut RaylibHandle, thread: &RaylibThread) {
        //Check for texture queries
        for query in self.image_queries.iter() {
            println!("Loading texture {:?}", query);

            {
                let provider = &mut self.provider;
                //Try to get the image from the provider
                if let Some(image) = provider.get_image(*query) {
                    //Get the texture from the image
                    let value = Some(context.load_texture_from_image(thread, image).unwrap());
                    //Insert the texture into the app's index/texture hash
                    self.textures.insert(*query, value);

                    if self.textures.len() >= 4 {
                        self.textures
                            .remove(&self.texture_loading_order.pop().unwrap());
                    }

                    self.texture_loading_order.insert(0, *query);
                }
            }
        }
    }

    #[inline]
    //Draw Application
    pub fn draw(&mut self, screen_rect: Rectangle, context: &mut RaylibDrawHandle) {
        if self.errors.len() > 0 {
            let err = self.errors.get(0).unwrap();

            let mut title_rect = screen_rect;
            let mut message_rect = screen_rect;
            let mut button_rect = screen_rect;

            title_rect.height = 30.0;
            message_rect.height -= 50.0;
            message_rect.y += 30.0;
            button_rect.y = screen_rect.y + screen_rect.height - 20.0;
            button_rect.height = 20.0;
            button_rect.x = screen_rect.x + screen_rect.width / 3.0;
            button_rect.width = screen_rect.width / 3.0;

            draw_text_centered(context, err.0.as_str(), title_rect, 14, Color::RED);
            draw_text_centered(context, err.1.as_str(), message_rect, 12, Color::RED);
            if context.gui_button(
                button_rect,
                Some(CString::new("Dismiss").unwrap().as_c_str()),
            ) {
                self.errors.remove(0);
            }
            return;
        }

        if self.handle_open_document(screen_rect, context) {
            return;
        }

        //Handle user input
        self.handle_input(context, &screen_rect);

        self.smoothed_scroll += (self.scroll - self.smoothed_scroll) * 0.3;

        //Draw borders(this is intended for debugging only)
        // context.draw_rectangle_lines_ex(screen_rect, 1, Color::DARKGRAY);

        //Unwrap a reference to the provider
        {
            let provider = &mut self.provider;
            //Store current chunk in cache
            self.current_chunk = if let Some(c) = provider.get_chunk(self.current_chunk_index) {
                Some(Chunk {
                    rect: c.rect,
                    texture_index: c.texture_index,
                })
            } else {
                None
            };
        }

        //If a chunk has been retrieved from the provider
        if let Some(chunk) = &self.current_chunk {
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
                //Width/Height Ratio
                let coeff = chunk.rect.height / chunk.rect.width;

                //Does the chunk fits in the screen?
                let chunk_real_height = screen_rect.width * coeff;
                let chunk_fits = chunk_real_height <= screen_rect.height;

                //Calculate the target rectangle for the texture
                let target_rect = Rectangle::new(
                    screen_rect.x,
                    if chunk_fits {
                        //If fits center the chunk vertically
                        (screen_rect.height - chunk_real_height) / 2.0
                    } else {
                        //Else start at screen_rect's beginning
                        0.0
                    } + screen_rect.y
                        + self.smoothed_scroll,
                    screen_rect.width,
                    screen_rect.width * coeff,
                );

                //Draw the texture
                context.draw_texture_pro(
                    t,
                    chunk.rect,
                    target_rect,
                    Vector2::zero(),
                    0f32,
                    Color::WHITE,
                )
            } else {
                //Draw a label with a "No Texture" message
                draw_text_centered(context, "No Texture", screen_rect, 14, Color::BLACK);
            }
        };

        //Y position for the indicator
        let y = screen_rect.y - 2.0 + screen_rect.height;
        //Available width
        let available_w = screen_rect.width - 10.0;
        //Offset between dots
        let offset = 10.0;
        //How many dots to draw
        let chunk_count = self.provider.chunk_count() as f32;

        //Actual width occupied by the dots
        let w = (offset) * chunk_count;
        //Does it fit in the available space?
        let it_fits = w < available_w;

        //If it does then draw the dots, else draw a fraction
        if it_fits {
            //Starting x offset for the dots
            let x_offset = (screen_rect.x + 5.0 + (available_w - w) / 2.0) as i32;

            for i in 0..(chunk_count as i32) {
                let (r, color) = if i == (self.current_chunk_index as i32) {
                    (4.0, Color::BLUE)
                } else {
                    (2.0, Color::GRAY)
                };
                context.draw_circle(x_offset + i * (offset) as i32, y as i32, r, color);
            }
        } else {
            //Current chunk (Applying 1-based index offset)
            let current = self.current_chunk_index + 1;
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
    fn handle_input(&mut self, context: &mut RaylibDrawHandle, screen_size: &Rectangle) {
        //Flag to signal that the user pressed next/prev or scrolled the image
        let mut something_changed = false;
        let mut real_size: Vector2 = Vector2::new(0.0, 0.0);

        // if context.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
        //     self.provider = None
        // }

        //Handle user scroll only if there is a chunk
        if let Some(chunk) = &self.current_chunk {
            //Calculate width/height ratio
            let coeff = chunk.rect.height / chunk.rect.width;
            //Calculate the chunk's screen-size
            real_size = Vector2::new(screen_size.width, screen_size.width * coeff);

            //If the chunk is taller than the screen then enable vertical scroll
            if real_size.y > screen_size.height {
                self.scroll += if context.is_key_down(KeyboardKey::KEY_DOWN) {
                    //Handle DOWN arrow
                    screen_size.height * -0.2
                } else if context.is_key_down(KeyboardKey::KEY_UP) {
                    //Handle UP arrow
                    screen_size.height * 0.2
                } else {
                    //If no keys were detected then try to get mousewheel's value
                    context.get_mouse_wheel_move() * 4.0
                };

                //Max possible offset
                let max_offset = -real_size.y + screen_size.height;

                //Keep scroll in bounds
                if self.scroll > 0.0 {
                    self.scroll = 0.0;
                } else if self.scroll < max_offset {
                    self.scroll = max_offset;
                }
            } else {
                //Reset scroll
                self.scroll = 0.0;
            }
        }

        //Initial chunk index
        let initial_chunk_index = self.current_chunk_index;

        //Check for simple next/prev events
        if context.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN)
            || context.is_key_pressed(KeyboardKey::KEY_RIGHT)
        {
            self.current_chunk_index += 1;
            something_changed = true;

            //Reset scroll position
            self.scroll = 0.0;
        } else if context.is_key_pressed(KeyboardKey::KEY_PAGE_UP)
            || context.is_key_pressed(KeyboardKey::KEY_LEFT)
        {
            if self.current_chunk_index > 0 {
                self.current_chunk_index -= 1;
            }
            something_changed = true;

            //Reset scroll position
            self.scroll = 0.0;
        }

        //Keep current_chunk into bounds
        if something_changed {
            {
                let provider = &self.provider;
                if self.current_chunk_index >= provider.chunk_count() {
                    self.current_chunk_index = provider.chunk_count() - 1;
                }
            }
        }

        if self.current_chunk_index > initial_chunk_index {
            self.smoothed_scroll = real_size.y;
        } else if self.current_chunk_index < initial_chunk_index {
            self.smoothed_scroll = -real_size.y;
        }
    }

    fn handle_open_document(
        &mut self,
        screen_rect: Rectangle,
        context: &mut RaylibDrawHandle,
    ) -> bool {
        {
            let provider = &self.provider;
            if provider.chunk_count() > 0 {
                return false;
            }
        }

        if self.recent_documents.len() == 0 {
            draw_text_centered(
                context,
                "No Recent documents",
                screen_rect,
                14,
                Color::BLACK,
            );
        } else {
            draw_text_centered(
                context,
                "Recent documents:",
                Rectangle::new(screen_rect.x, screen_rect.y, screen_rect.width, 20f32),
                14,
                Color::BLACK,
            );

            for i in 0..self.recent_documents.len() {
                if context.gui_button(
                    Rectangle::new(
                        screen_rect.x + 5.0,
                        20.0 + screen_rect.y + i as f32 * 20.0,
                        screen_rect.width - 10.0,
                        15.0,
                    ),
                    Some(
                        CString::new(self.recent_documents[i].as_str())
                            .unwrap()
                            .as_c_str(),
                    ),
                ) {
                    if !self.provider.open(&self.recent_documents[i]) {
                        self.add_error("Error", "Couldn't find a situable provider", None)
                    }
                }
            }
        }

        return true;
    }

    pub fn add_error(&mut self, title: &str, message: &str, callback: Option<fn()>) {
        self.errors
            .push((String::from(title), String::from(message), callback));
    }

    pub fn open_document(&mut self, path: String) -> Result<(), String> {
        self.recent_documents.push(path);

        self.write_recents();
        // self.add_error("Error", "Couldn't open document!", None);
        Ok(())
    }

    fn write_recents(&mut self) {
        self.recent_documents.sort();
        self.recent_documents.dedup();

        if let Ok(mut f) = OpenOptions::new()
            .write(true)
            .create(true)
            .open("recent.txt")
        {
            let all = self.recent_documents.join("\n");
            f.write_all(all.as_bytes())
                .expect("Error writing path to file");
        }
    }
}

fn draw_text_centered(
    context: &mut RaylibDrawHandle,
    text: &str,
    rect: Rectangle,
    font_size: i32,
    color: Color,
) {
    let text_width = measure_text(text, font_size);
    let x = rect.x + (rect.width - text_width as f32) / 2.0;
    let y = rect.y + (rect.height - font_size as f32) / 2.0;
    context.draw_text(text, x as i32, y as i32, font_size, color)
}
