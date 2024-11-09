use models::FileOrDirectory;
use serde::Deserialize;
use std::fs::File;
use crossterm::event::{ Event, KeyCode, KeyModifiers };
use crossterm::{ execute, style::{ Color, SetForegroundColor, ResetColor } };
use std::io::{ Read, stdout };
use serde_yaml;
use serde_json;
use cool_rust_input::{ CoolInput, CustomInput, set_terminal_line, KeyPressResult };
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

struct SupnoInput;
impl CustomInput for SupnoInput {
    fn get_offset(&mut self, _terminal_size: (u16, u16)) -> (u16, u16) {
        (0, 1)
    }
    fn before_draw_text(&mut self, _terminal_size: (u16, u16)) {
        let _ = execute!(stdout(), ResetColor);
    }
    fn after_draw_text(&mut self, _terminal_size: (u16, u16)) {
        let _ = execute!(stdout(), SetForegroundColor(Color::Green));
        set_terminal_line("[modifying wa.txt. press ctrl+x to save and exit]", 0, 0).unwrap();
    }
    fn handle_key_press(&mut self, key: &crossterm::event::Event) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if let KeyCode::Char(c) = key_event.code {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    if c == 'x' {
                        return KeyPressResult::Stop;
                    }
                }
            }
        }
        KeyPressResult::Continue
    }
}

#[tokio::main]
async fn main() {
    let config = load_config("config.yaml").expect("config bad :<, error");
    let data = api
        ::get_data(&config.bin_url, &config.x_master_key).await
        .expect("couldn't fetch >:(");
    //let data = "{\"supno\":\"yes\"}";
    let mut fs: models::FileSystem = serde_json
        ::from_str(&data)
        .expect("response json bad :<, error");

    let mut input = CoolInput::new(SupnoInput);
    let old_text = match fs.entries.get("supno").unwrap() {
        FileOrDirectory::File(data) => data.to_string(),
        FileOrDirectory::Directory(_) => String::from(""),
    };
    input.text = old_text.to_string();
    input.listen().unwrap();

    if input.text != old_text {
        fs.entries.insert("supno".to_string(), FileOrDirectory::File(input.text));
        let text = serde_json::to_string(&fs).expect("couldn't serialize json :<, error");
        api::set_data(text, &config.bin_url, &config.x_master_key).await.expect(
            "error setting data >:("
        );
    }
}
