use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use serde_yaml;
use serde_json;
use cool_rust_input::{ CoolInput, DefaultInput, CustomInput, set_terminal_line };
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

#[tokio::main]
async fn main() {
    //let config = load_config("config.yaml").expect("config bad :<, error");
    //let data = api::get_data(&config.bin_url, &config.x_master_key).await.expect("couldn't fetch >:(");
    let data = "{\"supno\":\"yes\"}";
    let fs: models::FileSystem = serde_json::from_str(&data).expect("response json bad :<, error");
    let text = serde_json::to_string(&fs).expect("couldn't serialize json :<, error");
    println!("{:#?}", text);

    let mut input = CoolInput::new(DefaultInput);
    input.listen().unwrap();
    //api::set_data(text, &config.bin_url, &config.x_master_key).await.expect(
    //    "error setting data >:("
    //);
}
