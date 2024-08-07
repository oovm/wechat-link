use crate::WeChatProcess;
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{borrow::Cow, fmt::Formatter};

pub struct UserInfo<'i> {
    // 昵称、账号、手机号、邮箱(过时)、key
    pub nick_name: Cow<'i, str>,
    // 账号
    pub user_name: Cow<'i, str>,
    // 手机号
    pub telephone: Cow<'i, str>,
    // 邮箱
    pub email: Cow<'i, str>,
    // 密钥
    pub key: Cow<'i, str>,
}

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
            user_name: self.find_info_by_offset(offsets.nick_name),
            telephone: self.find_info_by_offset(offsets.nick_name),
            email: self.find_info_by_offset(offsets.nick_name),
            key: self.find_info_by_offset(offsets.nick_name),
        }
    }
    fn find_info_by_offset(&self, offsets: usize) -> Cow<str> {
        let mut buffer = [0; 32];
        if !self.process.read_bytes(self.module.base_address() + offsets, buffer.as_mut_ptr(), buffer.len()) {
            eprintln!("wrong!")
        }
        String::from_utf8_lossy(&buffer)
    }
}

impl<'de> Deserialize for UserInfoOffset {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
impl<'i, 'de> Visitor for UserInfoOffsetVisitor {
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
            // 应昵称、账号、手机号、邮箱(过时)、key
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
