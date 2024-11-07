use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use serde_yaml;
use serde_json;
use std::process;
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
    let config = match load_config("config.yaml") {
        Ok(conf) => conf,
        Err(e) => {
            println!("config bad :<, error: {}", e);
            process::exit(1)
        }
    };
    //let data = api::get_data(&config.bin_url, &config.x_master_key).await.expect("couldn't fetch >:(");
    let data = "{\"supno\":\"yes\"}";
    let fs: models::FileSystem = serde_json::from_str(&data).expect("Failed to parse JSON");
    let text = serde_json::to_string(&fs).expect("wa");
    println!("{:#?}", text);
    api::set_data(text, &config.bin_url, &config.x_master_key).await.expect(
        "error setting data >:("
    );
}
