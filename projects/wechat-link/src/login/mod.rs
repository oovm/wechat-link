

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;

pub async fn get_uuid() -> Result<String, reqwest::Error> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("appid", "wx782c26e4c19acffb");
    params.insert("fun", "new");
    params.insert("lang", "zh_CN");
    params.insert("_", &SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string());
    params.insert("redirect_uri", "https://wx.qq.com/");

    let response = client.get("https://login.wx.qq.com/jslogin")
        .query(&params)
        .send()
        .await?;

    let body = response.text().await?;
    // let uuid = body.split("window.QRLogin.uuid = \"").nth(1)
    //     .unwrap_or_default()
    //     .split("\"").next()
    //     .unwrap_or_default()
    //     .to_string();

    Ok(body)
}
