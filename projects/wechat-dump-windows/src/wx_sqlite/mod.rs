use cbc::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use hmac::{Hmac, KeyInit, Mac};
use pbkdf2::pbkdf2_hmac_array;
use sha1::Sha1;

pub fn decrypt(path: &str, pkey: [u8; 32]) -> Result<Vec<u8>, anyhow::Error> {
    const IV_SIZE: usize = 16;
    const HMAC_SHA1_SIZE: usize = 20;
    const KEY_SIZE: usize = 32;
    const AES_BLOCK_SIZE: usize = 16;
    const SQLITE_HEADER: &str = "SQLite format 3";

    let mut buf = std::fs::read(path).unwrap();

    // 如果开头是 SQLITE_HEADER，说明不需要解密
    if buf.starts_with(SQLITE_HEADER.as_bytes()) {
        return Ok(buf);
    }

    let mut decrypted_buf: Vec<u8> = vec![];

    // 获取到文件开头的 salt，用于解密 key
    let salt = buf[..16].to_owned();
    // salt 异或 0x3a 得到 mac_salt， 用于计算HMAC
    let mac_salt: Vec<u8> = salt.to_owned().iter().map(|x| x ^ 0x3a).collect();

    unsafe {
        // 通过 pkey 和 salt 迭代64000次解出一个新的 key，用于解密
        // let pass = hex::decode(pkey)?;
        let pass = pkey;
        let key = pbkdf2_hmac_array::<Sha1, KEY_SIZE>(&pass, &salt, 64000);
        // 通过 key 和 mac_salt 迭代2次解出 mac_key
        let mac_key = pbkdf2_hmac_array::<Sha1, KEY_SIZE>(&key, &mac_salt, 2);

        // 开头是 sqlite 头
        decrypted_buf.extend(SQLITE_HEADER.as_bytes());
        decrypted_buf.push(0x00);

        // hash检验码对齐后长度 48，后面校验哈希用
        let mut reserve = IV_SIZE + HMAC_SHA1_SIZE;
        reserve = if (reserve % AES_BLOCK_SIZE) == 0 { reserve } else { ((reserve / AES_BLOCK_SIZE) + 1) * AES_BLOCK_SIZE };

        // 每页大小4096，分别解密
        const PAGE_SIZE: usize = 4096;
        let total_page = (buf.len() as f64 / PAGE_SIZE as f64).ceil() as usize;
        for cur_page in 0..total_page {
            let offset = if cur_page == 0 { 16 } else { 0 };
            let start: usize = cur_page * PAGE_SIZE;
            let end: usize = if (cur_page + 1) == total_page { start + buf.len() % PAGE_SIZE } else { start + PAGE_SIZE };

            // 搞不懂，这一堆0是干啥的，文件大小直接翻倍了
            if buf[start..end].iter().all(|&x| x == 0) {
                decrypted_buf.extend(&buf[start..]);
                break;
            }

            // 校验哈希
            type HamcSha1 = Hmac<Sha1>;

            let mut mac = HamcSha1::new_from_slice(&mac_key).unwrap();
            mac.update(&buf[start + offset..end - reserve + IV_SIZE]);
            mac.update(std::mem::transmute::<_, &[u8; 4]>(&(cur_page as u32 + 1)).as_ref());
            let hash_mac = mac.finalize().into_bytes().to_vec();

            let hash_mac_start_offset = end - reserve + IV_SIZE;
            let hash_mac_end_offset = hash_mac_start_offset + hash_mac.len();
            if hash_mac != &buf[hash_mac_start_offset..hash_mac_end_offset] {
                return Err(anyhow::anyhow!("Hash verification failed"));
            }

            // aes-256-cbc 解密内容
            type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

            let iv = &buf[end - reserve..end - reserve + IV_SIZE];
            decrypted_buf.extend(
                Aes256CbcDec::new(&key.into(), iv.into())
                    .decrypt_padded_mut::<NoPadding>(&mut buf[start + offset..end - reserve])
                    .map_err(anyhow::Error::msg)?,
            );
            decrypted_buf.extend(&buf[end - reserve..end]);
        }
    }

    Ok(decrypted_buf)
}
