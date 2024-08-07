use crate::WeChatProcess;
use indexmap::IndexMap;
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{fmt::Formatter, string::FromUtf8Error};

#[derive(Clone, Debug)]
pub struct UserInfo {
    // 昵称、账号、手机号、邮箱(过时)、key
    pub nick_name: String,
    // 账号
    pub user_name: String,
    // 手机号
    pub telephone: String,
    // 邮箱
    pub email: String,
    // 密钥
    pub key: [u8; 32],
}

#[derive(Default, Serialize)]
pub struct UserInfoOffset {
    // 昵称、账号、手机号、邮箱(过时)、key
    pub nick_name: usize,
    // 账号
    pub user_name: usize,
    // 手机号
    pub telephone: usize,
    // 邮箱
    pub email: usize,
    // 密钥
    pub key: usize,
}

impl WeChatProcess {
    pub fn find_user_by_offset(&self, offsets: UserInfoOffset) -> UserInfo {
        UserInfo {
            nick_name: self.find_info_by_offset(offsets.nick_name),
            user_name: self.find_info_by_offset(offsets.user_name),
            telephone: self.find_info_by_offset(offsets.telephone),
            email: self.find_info_by_offset(offsets.email),
            key: self.find_key_by_offset(offsets.key),
        }
    }
    fn find_info_by_offset(&self, offsets: usize) -> String {
        if offsets == 0 {
            return String::new();
        }

        let mut buffer = [0; 32];
        if !self.process.read_bytes(self.module.base_address() + offsets, buffer.as_mut_ptr(), buffer.len()) {
            eprintln!("wrong!")
        }
        let view = String::from_utf8_lossy(&buffer);
        return view.trim_end_matches(|c: char| !c.is_alphanumeric()).to_string();
        // return view.to_string();
    }
    fn find_key_by_offset(&self, offsets: usize) -> [u8; 32] {
        let mut buffer = [0; 8];
        if !self.process.read_bytes(self.module.base_address() + offsets, buffer.as_mut_ptr(), buffer.len()) {
            eprintln!("wrong!")
        }
        let ptr = usize::from_le_bytes(buffer);
        let mut buffer = [0; 32];
        if !self.process.read_bytes(ptr, buffer.as_mut_ptr(), buffer.len()) {
            eprintln!("wrong!")
        }
        return buffer
    }
}

impl UserInfoOffset {
    pub fn from_config(json: &str) -> IndexMap<String, UserInfoOffset> {
        serde_json::from_str(json).unwrap()
    }
    pub fn from_py_wx_dump(json: &str) -> IndexMap<String, UserInfoOffset> {
        serde_json::from_str(json).unwrap()
    }
}

impl<'de> Deserialize<'de> for UserInfoOffset {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut new = UserInfoOffset::default();
        let visitor = UserInfoOffsetVisitor { proxy: &mut new };
        deserializer.deserialize_any(visitor)?;
        Ok(new)
    }
}

impl<'i, 'de> Visitor<'de> for UserInfoOffsetVisitor<'i> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_seq<A>(self, mut list: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::with_capacity(5);
        while let Some(s) = list.next_element::<usize>()? {
            items.push(s)
        }
        match items.as_slice() {
            // 支持从 https://github.com/xaoyaoo/PyWxDump/blob/master/pywxdump/WX_OFFS.json 导入
            [a, b, c, d, e] => {
                self.proxy.nick_name = *a;
                self.proxy.user_name = *b;
                self.proxy.telephone = *c;
                self.proxy.email = *d;
                self.proxy.key = *e;
            }
            _ => {}
        }
        Ok(())
    }
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        todo!()
    }
}

struct UserInfoOffsetVisitor<'i> {
    proxy: &'i mut UserInfoOffset,
}
