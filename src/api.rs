use reqwest;

pub async fn get_data(
    url: String,
    x_master_key: String,
    x_access_key: String
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let res = client
        .get(url)
        .header("X-Master-Key", x_master_key)
        .header("X-Access-Key", x_access_key)
        .header("X-Bin-Meta", "false")
        .send().await?
        .text().await?;

    Ok(res)
}
