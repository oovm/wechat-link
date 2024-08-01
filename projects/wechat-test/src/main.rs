#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

use wechat_link::login::{get_qrcode, get_uuid};

#[tokio::main]
async fn main() {
    match get_uuid().await {
        Ok(uuid) => {
            get_qrcode(&uuid).await.unwrap();


        },
        Err(err) => eprintln!("Error: {}", err),
    }
}