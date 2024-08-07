#[derive(Copy, Clone, Debug)]
pub struct WeChatLogger {}

pub trait WeChatLink {
    fn login_scanned(&self) -> Result<(), reqwest::Error> {
        Ok(())
    }
}


impl WeChatLink for WeChatLogger {}