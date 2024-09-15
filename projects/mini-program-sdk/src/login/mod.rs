use std::collections::HashMap;
use reqwest::header::CONTENT_TYPE;
use crate::{MiniProgram, WechatSession};



impl MiniProgram {
    /// <https://developers.weixin.qq.com/miniprogram/dev/OpenApiDoc/user-login/code2Session.html>
    pub async fn code2session(&self, js_code: &str) -> Result<WechatSession, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("appid", self.app_id.as_ref());
        params.insert("secret", self.secret.as_ref());
        params.insert("js_code", js_code);
        params.insert("grant_type", "authorization_code");
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.weixin.qq.com/sns/jscode2session")
            .query(&params)
            .header(CONTENT_TYPE, "application/json")
            // .header(USER_AGENT, "your-app-name")
            .send()
            .await?
            .json::<WechatSession>()
            .await?;
        Ok(response)
    }
}
