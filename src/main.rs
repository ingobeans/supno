use models::{ FileOrDirectory, FileSystem };
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use crossterm::event::{ Event, KeyCode, KeyModifiers };
use crossterm::{ execute, style::{ Color, SetForegroundColor, ResetColor }, cursor, terminal };
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

struct EditFileInput;
impl CustomInput for EditFileInput {
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
pub fn set_terminal_line_dont_override(
    text: &str,
    x: usize,
    y: usize
) -> Result<(), std::io::Error> {
    execute!(stdout(), cursor::Hide)?;
    print!("\x1b[{};{}H{}", y + 1, x, text);
    Ok(())
}
struct TerminalInput {
    text: String,
    cwd: String,
    dirs: String,
    files: String,
}
impl CustomInput for TerminalInput {
    fn get_offset(&mut self, _terminal_size: (u16, u16)) -> (u16, u16) {
        (0, 3)
    }
    fn before_draw_text(&mut self, _terminal_size: (u16, u16)) {
        let _ = execute!(stdout(), ResetColor);
    }
    fn after_draw_text(&mut self, _terminal_size: (u16, u16)) {
        let _ = execute!(stdout(), SetForegroundColor(Color::Grey));
        set_terminal_line(&self.cwd, 0, 0).unwrap();
        set_terminal_line(&self.text, 0, 2).unwrap();
        let _ = execute!(stdout(), SetForegroundColor(Color::Green));
        set_terminal_line(&self.dirs, 0, 1).unwrap();
        let _ = execute!(stdout(), SetForegroundColor(Color::Blue));
        set_terminal_line_dont_override(&self.files, self.dirs.chars().count() + 1, 1).unwrap();
    }
    fn handle_key_press(&mut self, key: &crossterm::event::Event) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if let KeyCode::Enter = key_event.code {
                return KeyPressResult::Stop;
            }
        }
        KeyPressResult::Continue
    }
}

enum CommandResult {
    Ok,
    NotFound,
    BadArgs,
    Exit,
}
struct Supno {
    cwd: String,
    data: FileSystem,
    terminal_output: String,
}

impl Supno {
    fn new(data: FileSystem) -> Self {
        Supno {
            cwd: String::from("/"),
            data: data,
            terminal_output: String::from(""),
        }
    }
    fn move_to_dir(&mut self, name: &str) -> CommandResult {
        if name == ".." {
            let mut parts: Vec<&str> = self.cwd.split('/').collect();
            parts.retain(|&s| !s.is_empty());
            parts.pop();
            self.cwd = "/".to_string() + &parts.join("/");
            return CommandResult::Ok;
        }
        let current_dir = self.get_cwd_data();
        let new_dir = current_dir.get(name);
        if new_dir.is_some() {
            if let FileOrDirectory::Directory(_) = new_dir.unwrap() {
                if self.cwd == "/" {
                    self.cwd = String::from("/") + name;
                    return CommandResult::Ok;
                }
                self.cwd.insert_str(self.cwd.len(), &(String::from("/") + name));
                return CommandResult::Ok;
            }
        }
        CommandResult::BadArgs
    }
    fn get_cwd_data(&mut self) -> &HashMap<String, FileOrDirectory> {
        let mut parts: Vec<&str> = self.cwd.split('/').collect();
        parts.retain(|&s| !s.is_empty());
        let mut current_dir = &self.data.entries;
        for part in parts {
            if let FileOrDirectory::Directory(data) = current_dir.get(part).unwrap() {
                current_dir = &data;
            }
        }
        current_dir
    }
    fn handle_path(&mut self, name: &str) -> CommandResult {
        if name == ".." {
            self.move_to_dir(name);
        }
        let current_dir = self.get_cwd_data();
        let item = current_dir.get(name);

        if let Some(item) = item {
            // if the item specified is a directory, move to it
            // if its a file, edit it
            match item {
                FileOrDirectory::Directory(_) => {
                    self.move_to_dir(name);
                }
                FileOrDirectory::File(_) => {
                    self.open_file(name);
                }
            }
            return CommandResult::Ok;
        }
        CommandResult::BadArgs
    }
    fn remove_item(&mut self, name: &str) {}
    fn list_dir(&mut self) -> (String, String) {
        let mut dirs = String::new();
        let mut files = String::new();

        let current_dir = self.get_cwd_data();
        for (item, value) in current_dir {
            if let FileOrDirectory::Directory(_) = value {
                dirs += &(item.to_string() + &" ");
            } else {
                files += &(item.to_string() + &" ");
            }
        }
        (dirs, files)
    }
    fn open_file(&mut self, name: &str) -> CommandResult {
        let current_dir = self.get_cwd_data();
        let file = current_dir.get(name);
        if file.is_some() {
            if let FileOrDirectory::File(data) = file.unwrap() {
                let data = data.to_string();
                self.edit_data(data);
                return CommandResult::Ok;
            }
        }
        CommandResult::BadArgs
    }
    fn handle_command(&mut self, command: String) -> CommandResult {
        let mut args = command.split(' ');
        let keyword = args.next().unwrap_or("");
        let args: Vec<&str> = args.collect();
        match keyword {
            "" => {
                return CommandResult::Ok;
            }
            "cd" => {
                if args.len() != 1 {
                    return CommandResult::BadArgs;
                }
                return self.move_to_dir(args.first().unwrap());
            }
            "edit" => {
                if args.len() != 1 {
                    return CommandResult::BadArgs;
                }
                return self.open_file(args.first().unwrap());
            }
            "rm" => {
                if args.len() != 1 {
                    return CommandResult::BadArgs;
                }
                self.remove_item(args.first().unwrap());
            }
            "exit" => {
                return CommandResult::Exit;
            }
            _ => {
                return self.handle_path(keyword);
            }
        }

        CommandResult::Ok
    }
    fn edit_data(&mut self, data: String) -> String {
        let mut input = CoolInput::new(EditFileInput);
        input.text = data;
        input.listen().unwrap();
        input.text
    }
    fn listen_terminal(&mut self) {
        let mut input = CoolInput::new(TerminalInput {
            text: String::new(),
            cwd: String::new(),
            dirs: String::new(),
            files: String::new(),
        });
        loop {
            input.custom_input.text = self.terminal_output.to_string();
            input.custom_input.cwd = self.cwd.to_string();
            (input.custom_input.dirs, input.custom_input.files) = self.list_dir();

            input.text = String::new();
            input.cursor_x = 0;
            input.cursor_y = 0;
            input.listen();
            let result = self.handle_command(input.text);
            match result {
                CommandResult::Ok => {}
                CommandResult::BadArgs => {}
                CommandResult::NotFound => {}
                CommandResult::Exit => {
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let config = load_config("config.yaml").expect("config bad :<, error");
    //let data = api
    //    ::get_data(&config.bin_url, &config.x_master_key).await
    //    .expect("couldn't fetch >:(");
    let data = "{\"supno\":\"yes\",\"gnome\":{\"wa\":{},\"donkey\":\"horse\"}}";
    let mut fs: models::FileSystem = serde_json
        ::from_str(&data)
        .expect("response json bad :<, error");

    let mut supno = Supno::new(fs);
    supno.listen_terminal();

    //if false && input.text != old_text {
    //    fs.entries.insert("supno".to_string(), FileOrDirectory::File(input.text));
    //    let text = serde_json::to_string(&fs).expect("couldn't serialize json :<, error");
    //    api::set_data(text, &config.bin_url, &config.x_master_key).await.expect(
    //        "error setting data >:("
    //    );
    //}
}
