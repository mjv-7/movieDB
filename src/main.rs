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
use crate::modules::text_input::TextInput;
use crate::modules::listview::ListView;
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
    let mut txt_describtion = TextInput::new(100.0, 400.0, 350.0, 300.0, 25.0);
    let mut txt_names = TextInput::new(450.0, 150.0, 250.0, 60.0, 25.0);
    let mut txt_date = TextInput::new(100.0, 300.0, 250.0, 60.0, 25.0);
    let mut txt_title = TextInput::new(100.0, 150.0, 250.0, 60.0, 25.0);
    let mut lbl_display = Label::new(
        "", 
        450.0, 
        350.0, 
        50);
        lbl_display.with_colors(WHITE, Some(DARKGRAY));
        let items = vec!["Item 1".to_string(), "Item 2".to_string()];
        let mut list_view = ListView::new(&items, 800.0, 150.0,80 );
    let btn_add = TextButton::new(
        100.0, 
        850.0, 
        200.0, 
        60.0, 
        "Add", 
        BLUE, 
        GREEN, 
        30);
    let btn_delete = TextButton::new(
        350.0, 
        850.0, 
        200.0, 
        60.0, 
        "Delete", 
        BLUE, 
        GREEN, 
        30
    );
    let btn_update = TextButton::new(
        600.0, 
        850.0, 
        200.0, 
        60.0, 
        "Update", 
        BLUE, 
        GREEN, 
        30
    );
        /*lbl_display.set_text(txt_input.get_text());
        println!("Text: {}", txt_input.get_text());*/
    list_view.with_colors(BLACK, Some(WHITE), Some(RED));
    loop {
        clear_background(RED);
        lbl_display.draw();
        if btn_add.click() {
            lbl_display.set_text("Button Clicked!");
            println!("Button clicked!");
        }
        if btn_delete.click() {
        }
        if btn_update.click() {
        }
        txt_describtion.draw();
        txt_names.draw();
        txt_title.draw();
        txt_date.draw();
        list_view.draw();
        draw_grid(50.0, BLACK);
        next_frame().await;
    }
}
