use wechat_dump_windows::{UserInfoOffset, WeChatProcess};

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test() {
    for wechat in WeChatProcess::find().unwrap() {
        let offset = UserInfoOffset { nick_name: 93701080, user_name: 93702416, telephone: 93700888, email: 0, key: 93702352 };
        let user = wechat.find_user_by_offset(offset);
        println!("{user:#?}")
    }
}
