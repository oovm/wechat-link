use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;

pub async fn get_uuid() -> Result<String, reqwest::Error> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("appid", "wx782c26e4c19acffb");
    params.insert("fun", "new");
    params.insert("lang", "zh_CN");
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    params.insert("_", time.as_ref());
    params.insert("redirect_uri", "https://wx.qq.com/");

    let response = client.get("https://login.wx.qq.com/jslogin")
        .query(&params)
        .send()
        .await?;

    let body = response.text().await?;
    let text = body.rsplit("\"").nth(1).unwrap();

    Ok(text.to_string())
}

pub async fn get_qrcode(uuid: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("https://login.wx.qq.com/qrcode/{}", uuid);
    let params = vec![("t", "webwx")];

    let response = client.get(&url)
        .query(&params)
        .send()
        .await?;

    let mut file = File::create("qrcode.png").unwrap();
    let bytes = response.bytes().await?;
    file.write_all(&bytes).unwrap();

    Ok(())
}
