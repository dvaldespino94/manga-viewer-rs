use std::{cmp::min, collections::HashMap, ffi::CString};

use crate::{chunkprovider::metaprovider::MetaProvider, database::Database};
use raylib::prelude::*;

const DOTS_SHOW_TIMEOUT: f32 = 1.5;

use crate::{
    structs::{Chunk, ComicMetadata},
    traits::IChunkProvider,
};

#[derive(Debug)]
pub struct ApplicationFonts {
    fonts: Vec<Box<Font>>,
}

pub enum CardAction {
    None,
    OpenDocument,
    RemoveDocument,
}

#[allow(dead_code)]
impl ApplicationFonts {
    fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut fonts = Vec::new();

        fonts.push(Box::new(
            rl.load_font_ex(
                thread,
                "./fonts/Roboto-Light.ttf",
                14,
                FontLoadEx::Default(255),
            )
            .unwrap(),
        ));
        fonts.push(Box::new(
            rl.load_font_ex(
                thread,
                "./fonts/Roboto-Light.ttf",
                18,
                FontLoadEx::Default(255),
            )
            .unwrap(),
        ));
        fonts.push(Box::new(
            rl.load_font_ex(
                thread,
                "./fonts/Roboto-Light.ttf",
                22,
                FontLoadEx::Default(255),
            )
            .unwrap(),
        ));

        Self { fonts }
    }

    fn default(&self) -> &Box<Font> {
        self.fonts.get(0).unwrap()
    }
    fn large(&self) -> &Box<Font> {
        self.fonts.get(1).unwrap()
    }
    fn bold(&self) -> &Box<Font> {
        self.fonts.get(2).unwrap()
    }
}

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
    recent_documents: Vec<ComicMetadata>,
    //Texture indexes in the order they were loaded
    texture_loading_order: Vec<usize>,
    //Error messages
    pub errors: Vec<(String, String, Option<fn()>)>,
    //Fonts
    pub fonts: ApplicationFonts,
    //Metadata database
    pub db: Database,
    //Current Document Path
    pub current_document_path: Option<String>,
    pub logo_texture: Texture2D,
    pub recent_thumbs: Vec<Texture2D>,
    pub recent_thumbs_data: Vec<Vec<u8>>,
    show_dots_timeout: f32,
}

impl Application {
    /// Creates a new [`Application`].
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, logo_texture: Texture2D) -> Self {
        let provider = Box::new(MetaProvider::new());
        let mut db = Database::new();

        //Return a new application
        Self {
            current_chunk_index: 0,
            provider,
            current_chunk: None,
            image_queries: Vec::new(),
            textures: HashMap::new(),
            scroll: 0.0,
            smoothed_scroll: 0.0,
            recent_documents: db.get_recents(),
            texture_loading_order: Vec::new(),
            errors: Vec::new(),
            fonts: ApplicationFonts::new(rl, thread),
            db,
            current_document_path: None,
            logo_texture,
            recent_thumbs: Vec::new(),
            recent_thumbs_data: Vec::new(),
            show_dots_timeout: 5.0,
        }
    }

    pub fn load_textures(&mut self, context: &mut RaylibHandle, thread: &RaylibThread) {
        //Check for texture queries
        for query in self.image_queries.iter() {
            eprintln!("Loading texture {:?}", query);

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

    #[inline]
    //Draw Application
    pub fn draw(&mut self, screen_rect: Rectangle, context: &mut RaylibDrawHandle) {
        context.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_ARROW);
        if self.show_dots_timeout > 0.0 {
            self.show_dots_timeout -= 1.0 / (context.get_fps() as f32);
        }

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

            draw_text_centered(
                context,
                err.0.as_str(),
                title_rect,
                &self.fonts.large(),
                Color::RED,
            );
            draw_text_centered(
                context,
                err.1.as_str(),
                message_rect,
                &self.fonts.default(),
                Color::RED,
            );
            if context.gui_button(
                button_rect,
                Some(CString::new("Dismiss").unwrap().as_c_str()),
            ) {
                self.errors.remove(0);
            }
            return;
        }

        if self.lobby(screen_rect, context) {
            return;
        }

        //Handle user input
        self.handle_input(context, &screen_rect);

        self.smoothed_scroll += (self.scroll - self.smoothed_scroll) * 0.5;

        //Draw borders(this is intended for debugging only)
        // context.draw_rectangle_lines_ex(screen_rect, 1, Color::DARKGRAY);

        //Unwrap a reference to the provider
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

        //If a chunk has been retrieved from the provider
        if let Some(chunk) = &self.current_chunk {
            //Check if there is a texture already loaded from the provider
            let texture: &Option<Texture2D> = if self.textures.contains_key(&chunk.texture_index) {
                // eprintln!("Getting texture from cache!");
                //Unwrap the texture from the local hash
                self.textures.get(&chunk.texture_index).unwrap()
            } else {
                // eprintln!("Requesting image {}", chunk.texture_index);
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
                draw_text_centered(
                    context,
                    "No Texture",
                    screen_rect,
                    &self.fonts.large(),
                    Color::BLACK,
                );
            }
        };

        //Y position for the indicator
        let mut y = screen_rect.y - 12.0 + screen_rect.height;

        if self.show_dots_timeout < 0.2 {
            y = screen_rect.y + screen_rect.height + 70.0 * (0.2 - self.show_dots_timeout);
        }

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

        //Draws a progressbar if loading a batch of chunks when re-opening document or seeking
        if self.current_chunk_index > self.provider.chunk_count() {
            let percent =
                100.0 * self.provider.chunk_count() as f32 / self.current_chunk_index as f32;
            context.gui_progress_bar(
                Rectangle::new(
                    screen_rect.x + screen_rect.width / 3.0,
                    (screen_rect.y + screen_rect.height / 2.0) - 10.0,
                    screen_rect.width / 3.0,
                    20.0,
                ),
                Some(CString::new("Loading... ").unwrap().as_c_str()),
                Some(
                    CString::new(format!(" {:.1}%", percent))
                        .unwrap()
                        .as_c_str(),
                ),
                self.provider.chunk_count() as f32,
                0.0,
                self.current_chunk_index as f32,
            );
        }
        if it_fits {
            //If it does then draw the dots, else draw a fraction
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
            let message_extents = measure_text_ex(
                self.fonts.large() as &Font,
                fraction.as_str(),
                self.fonts.large().baseSize as f32,
                0.0,
            );

            //Starting x offset for fraction
            let x_offset = screen_rect.x + (available_w - message_extents.x) / 2.0;

            //Draw the fraction message
            context.draw_text_ex(
                self.fonts.default() as &Font,
                fraction.as_str(),
                Vector2::new(x_offset, y),
                self.fonts.default().baseSize as f32,
                0.0,
                Color::BLACK,
            );
        }
    }

    //Handle user input
    fn handle_input(&mut self, context: &mut RaylibDrawHandle, screen_size: &Rectangle) {
        //Flag to signal that the user pressed next/prev or scrolled the image
        let mut something_changed = false;
        let mut real_size: Vector2 = Vector2::new(0.0, 0.0);

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
                    screen_size.height * -0.1
                } else if context.is_key_down(KeyboardKey::KEY_UP) {
                    //Handle UP arrow
                    screen_size.height * 0.1
                } else {
                    //If no keys were detected then try to get mousewheel's value
                    context.get_mouse_wheel_move() * 2.0
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

            if !(self.provider.done_processing()
                && (self.current_chunk_index == self.provider.chunk_count()))
            {
                //Reset scroll position
                self.scroll = 0.0;
            }
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
            self.show_dots_timeout = DOTS_SHOW_TIMEOUT;
            let provider = &self.provider;
            if self.current_chunk_index >= provider.chunk_count() {
                self.current_chunk_index = provider.chunk_count() - 1;
            }
        }

        if self.current_chunk_index > initial_chunk_index {
            self.smoothed_scroll = real_size.y;
        } else if self.current_chunk_index < initial_chunk_index {
            self.smoothed_scroll = -real_size.y;
        }
    }

    fn draw_recent_card(
        &self,
        rect: Rectangle,
        context: &mut RaylibDrawHandle,
        metadata: Option<&ComicMetadata>,
        thumbnail: Option<&Texture2D>,
    ) -> CardAction {
        if metadata.is_none() {
            context.draw_rectangle_rounded_lines(
                Rectangle {
                    x: rect.x + 10.0,
                    y: rect.y + 10.0,
                    height: rect.height - 20.0,
                    width: rect.width - 20.0,
                },
                0.1,
                5,
                1,
                Color::LIGHTGRAY,
            );

            return CardAction::None;
        }

        let hovered = rect.check_collision_point_rec(context.get_mouse_position());
        let pressed = context.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON);

        if hovered {
            context.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_POINTING_HAND);
        }

        context.draw_rectangle_rounded_lines(
            rect,
            0.1,
            5,
            1,
            if hovered {
                if pressed {
                    Color::BLUE
                } else {
                    Color::DARKGRAY
                }
            } else {
                Color::LIGHTGRAY
            },
        );

        draw_text_centered(
            context,
            metadata.unwrap().title.as_str(),
            Rectangle::new(rect.x, rect.y + rect.height - 20.0, rect.width, 20.0),
            &self.fonts.default(),
            Color::BLACK,
        );
        let line_y = rect.y + rect.height - 20.0;
        context.draw_line(
            (rect.x) as i32,
            (line_y) as i32,
            (rect.x + rect.width) as i32,
            (line_y) as i32,
            Color::GRAY,
        );

        let source_rect = if hovered {
            Rectangle::new(
                0.0,
                0.0,
                self.logo_texture.width as f32,
                self.logo_texture.height as f32,
            )
        } else {
            Rectangle::new(
                -5.0,
                -5.0,
                (self.logo_texture.width + 10) as f32,
                (self.logo_texture.height + 10) as f32,
            )
        };

        context.draw_texture_pro(
            thumbnail.unwrap(),
            source_rect,
            Rectangle {
                height: rect.height - 20.0,
                ..rect
            },
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

        let delete_button_center = Vector2::new(rect.x + rect.width - 12.0, rect.y + 12.0);

        let delete_button_is_hovered = hovered
            && check_collision_point_circle(
                context.get_mouse_position(),
                delete_button_center,
                rect.width / 8.0,
            );

        if delete_button_is_hovered {
            context.draw_circle_v(delete_button_center, rect.width / 8.0, Color::RED.fade(0.5));
        }

        if context.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
            && delete_button_is_hovered
        {
            return CardAction::RemoveDocument;
        }

        if context.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON) && hovered {
            return CardAction::OpenDocument;
        }
        CardAction::None
    }

    fn lobby(&mut self, screen_rect: Rectangle, context: &mut RaylibDrawHandle) -> bool {
        if self.provider.chunk_count() > 0 {
            return false;
        }

        if self.recent_documents.len() == 0 {
            draw_text_centered(
                context,
                "No Recent documents",
                screen_rect,
                &self.fonts.large(),
                Color::BLACK,
            );
        } else {
            draw_text_centered(
                context,
                "Recent documents:",
                Rectangle::new(
                    screen_rect.x,
                    screen_rect.y + 20f32,
                    screen_rect.width,
                    20f32,
                ),
                self.fonts.bold(),
                Color::BLACK,
            );

            const MAX_RECENT_DOCUMENTS: usize = 8;
            const CARD_WIDTH: usize = 70;
            const CARD_HEIGHT: usize = 100;
            const CARD_SPACING: usize = 10;

            let cols = min(
                screen_rect.width as usize / (CARD_WIDTH + CARD_SPACING / 2),
                4,
            );
            let rows = min(
                MAX_RECENT_DOCUMENTS / cols as usize,
                screen_rect.height as usize / (CARD_HEIGHT + CARD_SPACING),
            );

            let cards_width = cols * CARD_WIDTH + (cols) * CARD_SPACING;
            let cards_height = rows * CARD_WIDTH + (rows) * CARD_SPACING;
            let x_offset = screen_rect.x + (screen_rect.width - cards_width as f32) / 2.0;
            let y_offset = screen_rect.y + (screen_rect.height - cards_height as f32) / 2.0;

            for row_index in 0..rows {
                for col_index in 0..cols {
                    let rect = Rectangle::new(
                        (x_offset as usize + (col_index * CARD_WIDTH + (col_index) * CARD_SPACING))
                            as f32,
                        (y_offset as usize + (row_index * CARD_HEIGHT + (row_index) * CARD_SPACING))
                            as f32,
                        CARD_WIDTH as f32,
                        CARD_HEIGHT as f32,
                    );

                    let index = col_index + (row_index * cols);

                    if index < self.recent_documents.len() {
                        let metadata = self.recent_documents.get(index).unwrap();

                        let thumbnail = Some(if metadata.thumbnail.is_none() {
                            &self.logo_texture
                        } else {
                            &self.recent_thumbs[index]
                        });

                        match self.draw_recent_card(rect, context, Some(metadata), thumbnail) {
                            CardAction::None => {}
                            CardAction::OpenDocument => {
                                let metadata = self.recent_documents.get(index).unwrap();
                                let path_copy = &metadata.path.to_string();
                                return self.open_document(path_copy).is_ok();
                            }
                            CardAction::RemoveDocument => {
                                self.recent_documents.remove(index);
                                self.db
                                    .save_recents(&self.recent_documents)
                                    .expect("Error saving recents");
                                return true;
                            }
                        }
                    } else {
                        self.draw_recent_card(rect, context, None, None);
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

    pub fn open_document(&mut self, path: &String) -> Result<(), String> {
        self.write_recents();

        self.close_document();

        let cached_chunks = self.db.chunks_for(path);

        match self.provider.open(path.as_str(), Some(cached_chunks)) {
            Err(error) => {
                self.add_error("Error", error.as_str(), None);

                return Err("Couldn't find a situable provider".to_string());
            }
            Ok(_) => {
                let metadata = if let Some(md) = self.db.metadata_for(path) {
                    md
                } else {
                    eprintln!("Using default metadata for {path}!");

                    let new_metadata = ComicMetadata {
                        title: String::from(path),
                        chunk_count: 0,
                        last_seen_chunk: 0,
                        path: String::from(path),
                        thumbnail: None,
                    };

                    //Save metadata for this document
                    self.db
                        .save_metadata(&Vec::from([&new_metadata]))
                        .expect("Error saving metadata");

                    new_metadata
                };

                self.current_chunk_index = metadata.last_seen_chunk;
                self.recent_documents.push(metadata.clone());

                if let Err(error) = self.db.save_recents(&self.recent_documents) {
                    eprintln!("Error occured saving recents: {:?}", error);
                }

                self.current_document_path = Some(metadata.path);

                self.recent_documents = self.db.get_recents();
            }
        }

        Ok(())
    }

    fn write_recents(&mut self) {
        if let Err(error) = self.db.save_recents(&self.recent_documents) {
            eprintln!("Error writing recents: {:?}", error);
        }
    }

    pub fn close_document(&mut self) {
        let path = if let Some(last_document) = &self.current_document_path {
            last_document.clone()
        } else {
            return;
        };

        let metadata = ComicMetadata {
            title: String::new(),
            chunk_count: self.provider.chunk_count(),
            last_seen_chunk: self.current_chunk_index,
            path,
            thumbnail: None,
        };
        self.db
            .save_metadata(&Vec::from([&metadata]))
            .expect("Error saving metadata");

        let all_chunks = self.all_chunks();
        self.db.save_chunk_cache(metadata.path, all_chunks);

        self.textures.clear();
        self.image_queries.clear();
        self.current_chunk_index = 0;
        self.texture_loading_order.clear();
        self.current_chunk = None;
        self.scroll = 0.0;
        self.smoothed_scroll = 0.0;
        self.provider.unload();
        self.current_document_path = None;
    }

    fn all_chunks(&mut self) -> Vec<Chunk> {
        (0..self.provider.chunk_count())
            .map(|index| {
                let c = self.provider.get_chunk(index);
                if c.is_some() {
                    *c.unwrap()
                } else {
                    Chunk {
                        rect: Rectangle::new(0.0, 0.0, 0.0, 0.0),
                        texture_index: 0,
                    }
                }
            })
            .filter(|x| x.rect.width > 0.0)
            .collect()
    }
}

fn draw_text_centered(
    context: &mut RaylibDrawHandle,
    text: &str,
    rect: Rectangle,
    font: &Font,
    color: Color,
) {
    let text_extents = measure_text_ex(font, text, font.baseSize as f32, 0.0);

    let x = rect.x + (rect.width - text_extents.x as f32) / 2.0;
    let y = rect.y + (rect.height - text_extents.y as f32) / 2.0;

    context.draw_text_ex(
        font,
        text,
        Vector2::new(x, y),
        font.baseSize as f32,
        0.0,
        color,
    )
}
