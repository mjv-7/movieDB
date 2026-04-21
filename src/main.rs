/*
By: <Your Name Here>
Date: 2026-03-30
Program Details: <Program Description Here>
*/

mod modules;
use crate::modules::database::{DatabaseClient, DatabaseTable, create_database_client, create_table_from_struct};
use crate::modules::grid::draw_grid;
use crate::modules::label::Label;
use crate::modules::listview::ListView;
use crate::modules::messagebox::MessageBox;
use crate::modules::text_button::TextButton;
use crate::modules::text_input::TextInput;
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
    let mut info_box = MessageBox::info("Information", "Operation completed successfully!");
    // info_box.show();  // Show it immediately or when needed

    let mut client = create_database_client();
    let mut txt_description = TextInput::new(100.0, 400.0, 350.0, 300.0, 25.0);
    txt_description.set_prompt("Enter the description...");
    txt_description.set_multiline(true);
    let mut txt_id = TextInput::new(450.0, 150.0, 250.0, 60.0, 25.0);
    txt_id.set_prompt("Enter the id...");
    let mut txt_date = TextInput::new(100.0, 300.0, 250.0, 60.0, 25.0);
    txt_date.set_prompt("Enter the year...");
    let mut txt_title = TextInput::new(100.0, 150.0, 250.0, 60.0, 25.0);
    txt_title.set_prompt("Enter the title...");
    let mut lbl_display = Label::new("", 450.0, 350.0, 50);
    lbl_display.with_colors(WHITE, Some(DARKGRAY));
    let items = vec!["Item 1".to_string(), "Item 2".to_string()];
    let mut list_view = ListView::new(&items, 800.0, 150.0, 80);
    list_view.with_max_visible_items(5);
    let btn_add = TextButton::new(100.0, 850.0, 200.0, 60.0, "Add", BLUE, GREEN, 30);
    let btn_delete = TextButton::new(350.0, 850.0, 200.0, 60.0, "Delete", BLUE, GREEN, 30);
    let btn_update = TextButton::new(600.0, 850.0, 200.0, 60.0, "Update", BLUE, GREEN, 30);
    let btn_search = TextButton::new(850.0, 850.0, 200.0, 60.0, "Search", BLUE, GREEN, 30);
    let btn_exit = TextButton::new(1100.0, 850.0, 200.0, 60.0, "Exit", BLUE, GREEN, 30);
    let btn_clear = TextButton::new(150.0, 700.0, 250.0, 60.0, "Clear", BLUE, GREEN, 30);
    list_view.with_colors(BLACK, Some(WHITE), Some(RED));
    async fn update_listview(list_view: &mut ListView, client: DatabaseClient) -> DatabaseClient {
        list_view.clear();

        let mut records: Vec<DatabaseTable> = Vec::new();
        let mut titles: Vec<String> = Vec::new();
        let matt = client.fetch_table("movies").await;
        if let Ok(result) = matt {
            records = result;
            for record in &records {
                titles.push(record.id.to_string() + ": " + &record.title.clone());
            }
        } else {
            println!("Error fetching records from database: {} ", matt.err().unwrap());
        }
        list_view.add_items(&titles);
        client
    }

    client = update_listview(&mut list_view, client).await;
    loop {
        clear_background(RED);
        lbl_display.draw();
        if btn_add.click() {
            // Insert a record (from user text input)

            let new_record = DatabaseTable {
                id: 0,
                title: txt_title.get_text(),
                year: txt_date.get_text().parse::<i32>().unwrap_or(0),
                description: txt_description.get_text(),
            };
            if let Ok(_id) = client.insert_record("movies", &new_record).await {
                client = update_listview(&mut list_view, client).await;
                // Inserted, id contains the new record's id
            } else {
                // Handle error
            }
        }
        if btn_delete.click() {
            // Delete a record by id (from user id input)
            match txt_id.get_text().trim().parse() {
                Ok(id) => {
                    if let Ok(_deleted_count) = client.delete_record_by_id("movies", id).await {
                        // deleted_count is the number of records deleted
                        client = update_listview(&mut list_view, client).await;
                    } else {
                        // Handle error
                    }
                }
                Err(e) => {
                    println!("Error parsing id: {}", e);
                }
            }
        }
        if btn_update.click() {
            // Update a record by struct (update all non-id fields)
            let id = txt_id.get_text().trim().parse::<i32>();
            let year = txt_date.get_text().trim().parse::<i32>();

            if let (Ok(id_parsed), Ok(year_parsed)) = (id, year) {
                let updated_record = DatabaseTable {
                    id: id_parsed,
                    title: txt_title.get_text(),
                    year: year_parsed,
                    description: txt_description.get_text(),
                };
                if let Ok(_updated_count) = client.update_record_by_struct("movies", &updated_record).await {
                    // updated_count is the number of records updated
                    client = update_listview(&mut list_view, client).await;
                } else {
                    // Handle error
                }
            } else{
                println!("Error parsing id or year");
            }
        }
        if btn_search.click() {
            let id = txt_id.get_text().parse().unwrap();
            if let Ok(Some(record)) = client.fetch_record_by_id::<DatabaseTable>("movies", id).await {
                println!("Successfully fetched record from database.");
                txt_title.set_text(record.title);
                txt_date.set_text(record.year.to_string());
                txt_description.set_text(record.description);
            } else if let Ok(None) = client.fetch_record_by_id::<DatabaseTable>("movies", id).await {
                println!("No record found with id {}", id);
            } else if let Err(err) = client.fetch_record_by_id::<DatabaseTable>("movies", id).await {
                println!("Error fetching record from database: {}", err);
            }
        }
        if btn_clear.click() {
            txt_id.set_text("");
            txt_title.set_text("");
            txt_date.set_text("");
            txt_description.set_text("");
        }
        if btn_exit.click() {
            break;
        }
        txt_description.draw();
        txt_id.draw();
        txt_title.draw();
        txt_date.draw();
        list_view.draw();
        info_box.draw();
        draw_grid(50.0, BLACK);
        next_frame().await;
    }
}
