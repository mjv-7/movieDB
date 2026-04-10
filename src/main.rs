/*
By: <Your Name Here>
Date: 2026-03-30
Program Details: <Program Description Here>
*/

mod modules;
use crate::modules::database::{DatabaseTable, create_database_client, create_table_from_struct};
use crate::modules::grid::draw_grid;
use crate::modules::label::Label;
use crate::modules::text_button::TextButton;
use macroquad::prelude::*;

/// Set up window settings before the app runs
fn window_conf() -> Conf {
    Conf {
        window_title: "movieDB".to_string(),
        window_width: 1440,
        window_height: 1080,
        fullscreen: false,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4, // MSAA: makes shapes look smoother
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let client = create_database_client();

    // Create table (call once at startup)
    if let Ok(_) = create_table_from_struct("messages").await {
        // Table created or already exists
    } else {
        // Handle error
    }

    let mut lbl_display = Label::new(
        "", 
        50.0, 
        100.0, 
        30);
        lbl_display.with_colors(WHITE, Some(DARKGRAY));
    let btn_add = TextButton::new(
        100.0, 
        200.0, 
        200.0, 
        60.0, 
        "Click Me", 
        BLUE, 
        GREEN, 
        30);
    loop {
        clear_background(WHITE);
        lbl_display.draw();
        if btn_add.click() {
            lbl_display.set_text("Button Clicked!");
            println!("Button clicked!");
        }
        draw_grid(50.0, BLACK);
        next_frame().await;
    }
}
