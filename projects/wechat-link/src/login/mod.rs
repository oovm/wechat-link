use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

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


async fn call_login_status(uuid: &str) -> Result<LoginStatus, reqwest::Error> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("tip", "1");
    params.insert("uuid", uuid);
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    params.insert("_", &time);
    params.insert("loginicon", "true");

    let response = client.get("https://login.wx.qq.com/cgi-bin/mmwebwx-bin/login")
        .query(&params)
        .send()
        .await?;

    let body = response.text().await?;
    println!("S: {body}");
    if body.starts_with("window.code=201") {
        Ok(LoginStatus::Waiting)
    } else if body.contains("window.code=400") {
        Ok(LoginStatus::Timeout)
    } else if body.contains("window.code=200") {
        let text = body.rsplit("\"").nth(1).unwrap();
        Ok(LoginStatus::Success {
            redirect_uri: text.to_string()
        })
    } else {
        Ok(LoginStatus::Waiting)
    }
}

#[derive(Clone, Debug)]
pub enum LoginStatus {
    Waiting,
    Scanned,
    Success {
        redirect_uri: String,
    },
    Timeout,
}


pub async fn poll_login_status(uuid: &str) -> Result<String, reqwest::Error> {
    let mut elapsed_time = Duration::from_secs(0);
    let max_wait_time = Duration::from_secs(60);

    loop {
        match call_login_status(uuid).await {
            Ok(LoginStatus::Waiting) => {
                println!("LoginStatus::Waiting")
            }
            Ok(LoginStatus::Scanned) => {
                println!("LoginStatus::Scanned")
            }
            Ok(LoginStatus::Timeout) => {
                println!("Maximum wait time of 10 seconds reached.");
                return Err(panic!());
            }
            Ok(LoginStatus::Success { redirect_uri }) => {
                return Ok(redirect_uri)
            }
            Err(_) => {}
        }

        sleep(Duration::from_millis(200)).await;
        elapsed_time += Duration::from_millis(200);

        if elapsed_time >= max_wait_time {
            println!("Maximum wait time of 10 seconds reached.");
            return Err(panic!());
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginInfo {
    pub ret: i32,
    pub message: String,
    pub skey: String,
    pub wxsid: String,
    pub wxuin: String,
    pub pass_ticket: String,
    pub isgrayscale: i32,
}

pub async fn get_login_info(redirect_uri: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("fun", "new");
    params.insert("version", "v2");
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    params.insert("_", &time);

    let response = client.get(redirect_uri)
        .query(&params)
        .send()
        .await?;

    let login_info = response.text().await?;

    if login_info.contains("暂不支持") {
        panic!("禁用")
    }

    Ok(login_info)
}
