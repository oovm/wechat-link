use std::borrow::Cow;
use serde::{Deserialize, Serialize};

mod login;

#[derive(Clone, Debug, Deserialize)]
pub struct MiniProgram {
    /// 小程序的 `app_id`
    pub app_id: Cow<'static, str>,
    /// 小程序的 `app_secret`
    pub secret: Cow<'static, str>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
// #[cfg_attr(feature = "poem-openapi", derive(poem_openapi::Object))]
pub struct WechatSession {
    #[serde(default, rename = "openid")]
    pub open_id: String,
    #[serde(default, rename = "unionid")]
    /// <https://developers.weixin.qq.com/miniprogram/dev/framework/open-ability/union-id.html>
    pub union_id: String,
    #[serde(default, rename = "session_key")]
    pub session_key: String,
    #[serde(rename = "errcode")]
    pub error_code: Option<i32>,
    #[serde(default, rename = "errmsg")]
    pub error_message: String,
}

impl WechatSession {
    pub fn as_result(&self) -> Result<Self, (i32, &str)> {
        match self.error_code {
            Some(0) | None => {
                Ok(self.clone())
            }
            Some(-1) => {
                Err((-1, "微信服务器系统繁忙, 请稍后再试"))
            }
            Some(40029) => {
                Err((40029, "无效的登录 code"))
            }
            Some(45011) => {
                Err((45011, "频率限制, 每个用户每分钟最多 100 次尝试"))
            }
            Some(40163) => {
                Err((40163, "该登录 code 已被使用"))
            }
            Some(40226) => {
                Err((40226, "高风险用户, 已被微信禁止登录"))
            }
            Some(i) => {
                Err((i, &self.error_message))
            }
        }
    }
}