use raylib::drawing::{RaylibDraw, RaylibDrawHandle};
use raylib::math::Rectangle;
use raylib::color::Color;

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
