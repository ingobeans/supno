use reqwest;
use super::models;

pub async fn get_data(url: &String, x_master_key: &String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let res = client
        .get(url)
        .header("X-Master-Key", x_master_key)
        .header("X-Bin-Meta", "false")
        .send().await?
        .text().await?;

    Ok(res)
}

pub async fn set_data(
    data: String,
    url: &String,
    x_master_key: &String
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .put(url)
        .header("X-Master-Key", x_master_key)
        .header("X-Bin-Meta", "false")
        .header("Content-Type", "application/json")
        .body(data)
        .send().await?
        .text().await?;
    Ok(())
}
