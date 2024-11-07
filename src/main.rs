use std::fs::File;
use std::io::Read;
use serde_yaml;
use std::process;
mod api;

#[derive(Debug, serde::Deserialize)]
struct Config {
    x_master_key: String,
    x_access_key: String,
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
    let data = match api::get_data(config.bin_url, config.x_master_key, config.x_access_key).await {
        Ok(data) => data,
        Err(e) => {
            println!("couldn't fetch :<, error: {}", e);
            process::exit(1)
        }
    };
    println!("{}", data);
}
