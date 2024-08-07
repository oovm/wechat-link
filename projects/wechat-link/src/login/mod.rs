use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::{Client, Error};
use reqwest::header::{REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

mod desktop;

const USER_AGENT_DEFAULT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36";

const UOS_PATCH_CLIENT_VERSION: &'static str = "2.0.0";
const UOS_PATCH_EXTSPAM: &'static str = include_str!("uos.txt");

fn wx_time() -> (String, String) {
    let local_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    return ((-local_time / 1579).to_string(), local_time.to_string());
}

pub async fn get_uuid() -> Result<String, reqwest::Error> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("appid", "wx782c26e4c19acffb");
    params.insert("fun", "new");
    params.insert("lang", "zh_CN");
    let (rt, _t) = wx_time();
    params.insert("r", &rt);
    params.insert("_", &_t);
    params.insert("redirect_uri", "https://wx.qq.com/cgi-bin/mmwebwx-bin/webwxnewloginpage?mod=desktop");
    let response = client.get("https://login.wx.qq.com/jslogin")
        .header(USER_AGENT, USER_AGENT_DEFAULT)
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
        .header(USER_AGENT, USER_AGENT_DEFAULT)
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
    let (rt, _t) = wx_time();
    params.insert("r", &rt);
    params.insert("_", &_t);
    params.insert("loginicon", "true");

    let response = client.get("https://login.wx.qq.com/cgi-bin/mmwebwx-bin/login")
        .header(USER_AGENT, USER_AGENT_DEFAULT)
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
    let response = Client::new()
        .get(redirect_uri)
        .header(USER_AGENT, USER_AGENT_DEFAULT)
        .header("client-version", UOS_PATCH_CLIENT_VERSION)
        .header("extspam", UOS_PATCH_EXTSPAM)
        .header(REFERER, "https://wx.qq.com/?&lang=zh_CN&target=t")
        // .query(&params)
        .send()
        .await?;
    let login_info = response.text().await?;
    Ok(login_info)
}
