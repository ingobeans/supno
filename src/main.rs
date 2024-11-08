#![allow(unused)]

use serde::Deserialize;
use std::fs::File;
use std::io::{ stdout, Read };
use serde_yaml;
use serde_json;
use crossterm::event::{ self, DisableFocusChange, Event, KeyCode, KeyEventKind };
use crossterm::{
    execute,
    cursor,
    terminal,
    style::{ Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor },
};
use std::io::{ self, Write };
use std::time::Duration;
use std::cmp;
mod models;
mod api;

#[derive(Debug, Deserialize)]
struct Config {
    x_master_key: String,
    bin_url: String,
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

fn update_cursor(document: &String, cursor_x: usize, cursor_y: usize) {
    execute!(
        stdout(),
        cursor::MoveTo(cursor_x.try_into().unwrap(), cursor_y.try_into().unwrap())
    ).unwrap();
}

fn insert_string(document: String, c: char, x: usize, y: usize) -> String {
    let mut new = String::new();
    let mut cur_x = 0;
    let mut cur_y = 0;

    if document.len() > 0 {
        for (i, char) in document.chars().enumerate() {
            cur_x += 1;
            if char == '\n' {
                cur_y += 1;
                cur_x = 0;
            }
            new.insert(new.len(), char);
            if cur_x == x && cur_y == y {
                new.insert(new.len(), c);
            }
        }
    } else {
        new.insert(0, c);
    }
    new
}

fn remove_character(document: String, x: usize, y: usize) -> (String, usize, usize) {
    let mut new = String::new();
    let mut cur_x = 0;
    let mut cur_y = 0;

    let mut cursor_x = x;
    let mut cursor_y = y;

    if x == 0 {
        cursor_y -= 1;
        cursor_x = document.lines().nth(cursor_y).unwrap().len();
    } else {
        cursor_x -= 1;
    }

    if document.len() > 0 {
        for (i, char) in document.chars().enumerate() {
            cur_x += 1;
            if char == '\n' {
                cur_y += 1;
                cur_x = 0;
            }
            if cur_x != x || cur_y != y {
                new.insert(new.len(), char);
            }
        }
    } else {
        "";
    }
    (new, cursor_x, cursor_y)
}

fn update_text(document: &String, screen_buffer: &mut Vec<String>) {
    let new_lines: Vec<&str> = document.lines().collect();
    let size = terminal::size().unwrap();
    let height = size.1;
    let width = size.0;
    let lines = document.lines().count();

    for y in 0..height {
        if y < (lines as u16) {
            let line = document
                .lines()
                .nth(y as usize)
                .unwrap();
            print!(
                "\x1b[{};0H{}",
                y + 1,
                String::from(line) + &" ".repeat((width - (line.len() as u16)).into())
            );
        } else {
            print!("\x1b[{};0H{}", y + 1, " ".repeat(width as usize));
        }
    }

    //print!("{}", document);
    io::stdout().flush().unwrap();
}

#[tokio::main]
async fn main() {
    //let config = load_config("config.yaml").expect("config bad :<, error");
    //let data = api::get_data(&config.bin_url, &config.x_master_key).await.expect("couldn't fetch >:(");
    let data = "{\"supno\":\"yes\"}";
    let fs: models::FileSystem = serde_json::from_str(&data).expect("response json bad :<, error");
    let text = serde_json::to_string(&fs).expect("couldn't serialize json :<, error");
    println!("{:#?}", text);
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        SetForegroundColor(Color::Blue)
    ).unwrap();
    //api::set_data(text, &config.bin_url, &config.x_master_key).await.expect(
    //    "error setting data >:("
    //);
    let mut document = String::new();
    let mut cursor_x: usize = 0;
    let mut cursor_y: usize = 0;

    let mut screen_buffer: Vec<String> = vec![];

    update_text(&document, &mut screen_buffer);
    execute!(
        stdout(),
        cursor::MoveTo(cursor_x.try_into().unwrap(), cursor_y.try_into().unwrap())
    ).unwrap();
    loop {
        if event::poll(Duration::from_millis(50)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key_event) => {
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                document = insert_string(document, c, cursor_x, cursor_y);
                                cursor_x += 1;
                                update_text(&document, &mut screen_buffer);
                                update_cursor(&document, cursor_x, cursor_y);
                            }
                            KeyCode::Enter => {
                                document = insert_string(document, '\n', cursor_x, cursor_y);
                                cursor_x = 0;
                                cursor_y += 1;
                                update_text(&document, &mut screen_buffer);
                                update_cursor(&document, cursor_x, cursor_y);
                            }
                            KeyCode::Backspace => {
                                if document.len() > 0 {
                                    (document, cursor_x, cursor_y) = remove_character(
                                        document,
                                        cursor_x,
                                        cursor_y
                                    );
                                    update_text(&document, &mut screen_buffer);
                                    update_cursor(&document, cursor_x, cursor_y);
                                }
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Up => {
                                if cursor_y > 0 {
                                    cursor_y -= 1;
                                    cursor_x = cmp::min(
                                        document.lines().nth(cursor_y).unwrap().len(),
                                        cursor_x
                                    );
                                }
                                update_cursor(&document, cursor_x, cursor_y);
                            }
                            KeyCode::Down => {
                                if document.lines().count() > 0 {
                                    if cursor_y < document.lines().count() - 1 {
                                        cursor_y += 1;
                                        cursor_x = cmp::min(
                                            document.lines().nth(cursor_y).unwrap().len(),
                                            cursor_x
                                        );
                                        update_cursor(&document, cursor_x, cursor_y);
                                    }
                                }
                            }
                            KeyCode::Left => {
                                if cursor_x > 0 || cursor_y != 0 {
                                    if cursor_x > 0 {
                                        cursor_x -= 1;
                                    } else {
                                        cursor_y -= 1;
                                        cursor_x = document.lines().nth(cursor_y).unwrap().len();
                                    }
                                }
                                update_cursor(&document, cursor_x, cursor_y);
                            }
                            KeyCode::Right => {
                                if document.lines().count() > 0 {
                                    if
                                        cursor_y != document.lines().count() - 1 ||
                                        cursor_x < document.lines().nth(cursor_y).unwrap().len()
                                    {
                                        if
                                            cursor_x !=
                                            document.lines().nth(cursor_y).unwrap().len()
                                        {
                                            cursor_x += 1;
                                        } else {
                                            cursor_y += 1;
                                            cursor_x = 0;
                                        }
                                        update_cursor(&document, cursor_x, cursor_y);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => (),
            }
        }
    }
    execute!(
        stdout(),
        ResetColor,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    ).unwrap();
}
