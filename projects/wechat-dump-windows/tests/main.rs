use std::fs::File;
use std::io::Write;
use wechat_dump_windows::{decrypt, UserInfoOffset, WeChatProcess};

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test() {
    for wechat in WeChatProcess::find().unwrap() {
        let offset = UserInfoOffset { nick_name: 93701080, user_name: 93702416, telephone: 93700888, email: 0, key: 93702352 };
        let user = wechat.find_user_by_offset(offset);
        println!("{user:#?}");
        let output = decrypt(

            r#"C:\Users\Aster\Documents\WeChat Files\wxid_hjsdcf4uqrkg22\Msg\MicroMsg.db"#,
            user.key,

        ).unwrap();
        let mut sq = File::create(r#"C:\Users\Aster\Documents\WeChat Files\wxid_hjsdcf4uqrkg22\Msg\MicroMsg.sqlite"#).unwrap();
        sq.write_all(&output).unwrap()


    }
}
