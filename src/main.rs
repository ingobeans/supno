use models::{ FileOrDirectory, FileSystem };
use serde::Deserialize;
use std::collections::HashMap;
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

fn load_config(path: &str) -> Result<Config, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = serde_yaml
        ::from_str(&contents)
        .map_err(|_| { std::io::Error::new(std::io::ErrorKind::Other, "Couldn't pass yaml") })?;

    Ok(config)
}

struct EditFileInput {
    file_name: String,
    should_save_file: bool,
    should_continue: bool,
}
impl CustomInput for EditFileInput {
    fn get_offset(&mut self, _terminal_size: (u16, u16), _current_text: String) -> (u16, u16) {
        (0, 3)
    }
    fn get_size(&mut self, terminal_size: (u16, u16), _current_text: String) -> (u16, u16) {
        (terminal_size.0, terminal_size.1 - 3)
    }
    fn before_draw_text(&mut self, _terminal_size: (u16, u16), _current_text: String) {
        let _ = execute!(stdout(), ResetColor);
    }
    fn after_draw_text(&mut self, _terminal_size: (u16, u16), _current_text: String) {
        let _ = execute!(stdout(), SetForegroundColor(Color::Blue));
        let header = "[".to_string() + &self.file_name + "]";
        set_terminal_line(&header, 0, 0, true).unwrap();
        set_terminal_line(
            "ctrl+s to save | ctrl+q to exit | ctrl+x to save and exit",
            0,
            1,
            true
        ).unwrap();
    }
    fn handle_key_press(
        &mut self,
        key: &crossterm::event::Event,
        _current_text: String
    ) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if let KeyCode::Char(c) = key_event.code {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    if c == 'x' {
                        self.should_save_file = true;
                        return KeyPressResult::Stop;
                    }
                    if c == 'q' {
                        self.should_save_file = false;
                        return KeyPressResult::Stop;
                    }
                    if c == 's' {
                        self.should_save_file = true;
                        self.should_continue = true;
                        return KeyPressResult::Stop;
                    }
                }
            }
        }
        KeyPressResult::Continue
    }
}
struct TerminalInput {
    error_message: String,
    cwd: String,
    dirs: String,
    files: String,
    items: Vec<String>,
    current_autocomplete: Option<String>,
    should_quit: bool,
    should_back: bool,
}
impl TerminalInput {
    fn autocomplete_input(&mut self, current_input: String) -> Option<String> {
        if current_input.is_empty() {
            return None;
        }

        let mut items = self.items.clone();
        items.sort_by_key(|item| item.len());
        for item in &self.items {
            if item == &current_input {
                return None;
            }
            if item.starts_with(&current_input) {
                return Some(item.trim_start_matches(&current_input).to_string());
            }
        }
        None
    }
}
impl CustomInput for TerminalInput {
    fn get_offset(&mut self, _terminal_size: (u16, u16), _current_text: String) -> (u16, u16) {
        (0, 3)
    }
    fn get_size(&mut self, terminal_size: (u16, u16), _current_text: String) -> (u16, u16) {
        (terminal_size.0, terminal_size.1 - 3)
    }
    fn before_draw_text(&mut self, _terminal_size: (u16, u16), _current_text: String) {
        let _ = execute!(stdout(), ResetColor);
    }
    fn after_draw_text(&mut self, _terminal_size: (u16, u16), current_text: String) {
        let _ = execute!(stdout(), SetForegroundColor(Color::Grey));
        set_terminal_line(&self.cwd, 0, 0, true).unwrap();
        let _ = execute!(stdout(), SetForegroundColor(Color::Green));
        set_terminal_line(&self.dirs, 0, 1, true).unwrap();
        let _ = execute!(stdout(), SetForegroundColor(Color::Blue));
        set_terminal_line(&self.files, self.dirs.chars().count() + 1, 1, false).unwrap();
        let _ = execute!(stdout(), SetForegroundColor(Color::Red));
        set_terminal_line(&self.error_message, 0, 2, true).unwrap();

        let _ = execute!(stdout(), SetForegroundColor(Color::DarkGrey));
        let input_length = current_text.chars().count();
        let autocomplete = self.autocomplete_input(current_text);
        if let Some(autocomplete) = autocomplete {
            let _ = set_terminal_line(&autocomplete, input_length, 3, false);
            self.current_autocomplete = Some(autocomplete.to_string());
        } else {
            self.current_autocomplete = None;
        }
    }
    fn handle_key_press(
        &mut self,
        key: &crossterm::event::Event,
        _current_text: String
    ) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if key_event.kind == crossterm::event::KeyEventKind::Press {
                if let KeyCode::Enter = key_event.code {
                    return KeyPressResult::Stop;
                }
                if let KeyCode::Esc = key_event.code {
                    self.should_back = true;
                    return KeyPressResult::Stop;
                }
                if let KeyCode::Char(c) = key_event.code {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        if c == 'x' {
                            self.should_back = true;
                            return KeyPressResult::Stop;
                        }
                    }
                }
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
    error_message: String,
    has_been_modified: bool,
}

impl Supno {
    fn new(data: FileSystem) -> Self {
        Supno {
            cwd: "/".to_string(),
            data: data,
            error_message: "".to_string(),
            has_been_modified: false,
        }
    }
    fn move_to_dir(&mut self, name: &str) -> CommandResult {
        if name.chars().all(|c| c == '.') {
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
                    self.cwd = "/".to_string() + name;
                    return CommandResult::Ok;
                }
                self.cwd.insert_str(self.cwd.len(), &("/".to_string() + name));
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
    fn get_cwd_data_mut(&mut self) -> &mut HashMap<String, FileOrDirectory> {
        let mut parts: Vec<&str> = self.cwd.split('/').collect();
        parts.retain(|&s| !s.is_empty());
        let mut current_dir = &mut self.data.entries;
        for part in parts {
            if let FileOrDirectory::Directory(data) = current_dir.get_mut(part).unwrap() {
                current_dir = data;
            } else {
                panic!();
            }
        }
        current_dir
    }
    fn handle_path(&mut self, name: &str) -> CommandResult {
        if name.chars().all(|c| c == '.') {
            self.move_to_dir(name);
            return CommandResult::Ok;
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
        CommandResult::NotFound
    }
    fn remove_item(&mut self, name: &str) -> CommandResult {
        let current_dir = self.get_cwd_data_mut();

        let item = current_dir.get(name);
        if item.is_some() {
            current_dir.remove(name);
            self.has_been_modified = true;
            return CommandResult::Ok;
        }
        CommandResult::BadArgs
    }
    fn list_dir(&mut self) -> (Vec<String>, String, String) {
        let mut dirs = String::new();
        let mut files = String::new();
        let mut items: Vec<String> = Vec::new();

        let current_dir = self.get_cwd_data();
        for (item, value) in current_dir {
            if let FileOrDirectory::Directory(_) = value {
                dirs += &(item.to_string() + &" ");
            } else {
                files += &(item.to_string() + &" ");
            }
            items.push(item.to_string());
        }
        (items, dirs, files)
    }
    fn open_file(&mut self, name: &str) -> CommandResult {
        let current_dir = self.get_cwd_data();
        let file = current_dir.get(name);
        if let Some(FileOrDirectory::File(data)) = file {
            let data = data.to_string();
            let old_data = data.clone();

            let mut input = CoolInput::new(EditFileInput {
                file_name: name.to_string(),
                should_save_file: false,
                should_continue: false,
            });
            input.text = data;
            let mut should_continue = true;
            input.pre_listen().unwrap();
            input.render().unwrap();
            while should_continue {
                input.custom_input.should_save_file = false;
                input.custom_input.should_continue = false;
                input.listen_quiet().unwrap();
                should_continue = input.custom_input.should_continue;
                let new = input.text.to_string();
                if input.custom_input.should_save_file {
                    self.has_been_modified = self.has_been_modified || old_data != new;

                    let current_dir = self.get_cwd_data_mut();
                    current_dir.insert(name.to_string(), FileOrDirectory::File(new.to_string()));
                }
            }
            input.post_listen().unwrap();

            return CommandResult::Ok;
        }

        CommandResult::BadArgs
    }
    fn create_file(&mut self, name: &str) -> CommandResult {
        let current_dir = self.get_cwd_data_mut();
        if current_dir.get(name).is_none() {
            current_dir.insert(name.to_string(), FileOrDirectory::File(String::new()));
            self.open_file(name);
            return CommandResult::Ok;
        }
        CommandResult::BadArgs
    }
    fn create_dir(&mut self, name: &str) -> CommandResult {
        let current_dir = self.get_cwd_data_mut();
        if current_dir.get(name).is_none() {
            current_dir.insert(name.to_string(), FileOrDirectory::Directory(HashMap::new()));
            self.has_been_modified = true;
            self.move_to_dir(name);
            return CommandResult::Ok;
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
                return self.remove_item(args.first().unwrap());
            }
            "n" | "new" => {
                if args.len() != 1 {
                    return CommandResult::BadArgs;
                }
                return self.create_file(args.first().unwrap());
            }
            "d" | "mkdir" => {
                if args.len() != 1 {
                    return CommandResult::BadArgs;
                }
                return self.create_dir(args.first().unwrap());
            }
            "abort" => {
                self.has_been_modified = false;
                return CommandResult::Exit;
            }
            "exit" => {
                return CommandResult::Exit;
            }
            "ok" => {
                return CommandResult::Ok;
            }
            _ => {
                return self.handle_path(keyword);
            }
        }
    }

    fn listen_terminal(&mut self) {
        let mut input = CoolInput::new(TerminalInput {
            error_message: String::new(),
            cwd: String::new(),
            dirs: String::new(),
            files: String::new(),
            items: Vec::new(),
            current_autocomplete: None,
            should_quit: false,
            should_back: false,
        });

        input.pre_listen().unwrap();
        loop {
            input.custom_input.error_message = self.error_message.to_string();
            input.custom_input.cwd = self.cwd.to_string();
            (input.custom_input.items, input.custom_input.dirs, input.custom_input.files) =
                self.list_dir();

            input.text = String::new();
            input.cursor_x = 0;
            input.cursor_y = 0;
            input.custom_input.should_back = false;
            input.render().unwrap();
            input.listen_quiet().unwrap();
            if input.custom_input.should_quit {
                break;
            }
            if input.custom_input.should_back {
                if self.cwd == "/" {
                    break;
                }
                self.move_to_dir("..");
                continue;
            }
            let result = self.handle_command(input.text.to_string());
            match result {
                CommandResult::Ok => {
                    self.error_message = String::new();
                }
                CommandResult::BadArgs => {
                    self.error_message = "bad args".to_string();
                }
                CommandResult::NotFound => {
                    if let Some(ref autocomplete) = input.custom_input.current_autocomplete {
                        let full = input.text.to_string() + &autocomplete.to_string();
                        self.handle_path(&full);
                    } else {
                        self.error_message =
                            "unknown command or nonexisting file/directory".to_string();
                    }
                }
                CommandResult::Exit => {
                    break;
                }
            }
        }
        input.post_listen().unwrap();
    }
}

#[tokio::main]
async fn main() {
    let config = load_config("config.yaml").expect("config bad :<, error");
    let data = api
        ::get_data(&config.bin_url, &config.x_master_key).await
        .expect("couldn't fetch >:(");
    //let data = "{\".supno\":\"yes\",\"gnome\":{\"wa\":{},\"donkey\":\"horse\"}}";
    let mut fs: models::FileSystem = serde_json
        ::from_str(&data)
        .expect("response json bad :<, error");
    fs.entries.remove(".supno");
    let mut supno = Supno::new(fs);
    supno.listen_terminal();
    let supno_modified = supno.has_been_modified == true;
    if supno_modified {
        let mut data = supno.data;
        data.entries.insert(".supno".to_string(), FileOrDirectory::File("yes".to_string()));
        let text = serde_json::to_string(&data).expect("couldn't serialize json :<, error");
        api::set_data(text, &config.bin_url, &config.x_master_key).await.expect(
            "error setting data >:("
        );
        println!("saved to cloud!");
    }
}
