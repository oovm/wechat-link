use aes::{cbc::Cbc, Aes128, BlockMode, NewBlockCipher};
use hmac::{Hmac, Mac, NewMac};
use pbkdf2::pbkdf2;
use rand::Rng;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

const DEFAULT_ITER: u32 = 1000;
const KEY_SIZE: usize = 32; // 256 bits
const DEFAULT_PAGESIZE: usize = 4096;
const SQLITE_FILE_HEADER: &str = "SQLite format 3\0";

pub fn decrypt(key: &str, db_path: &str, out_path: &str) -> Result<Vec<bool>, String> {
    if !Path::new(db_path).exists() {
        return Err(format!("[-] db_path:'{}' File not found!", db_path));
    }
    if !Path::new(out_path).parent().unwrap().exists() {
        return Err(format!("[-] out_path:'{}' File not found!", out_path));
    }
    if key.len() != 64 {
        return Err(format!("[-] key:'{}' Error!", key));
    }

    let password = hex::decode(key.trim()).map_err(|_| "Invalid hex key")?;
    let mut file = fs::File::open(db_path).map_err(|e| e.to_string())?;
    let mut blist = Vec::new();
    file.read_to_end(&mut blist).map_err(|e| e.to_string())?;

    let salt = &blist[..16];
    let mut byte_key = vec![0u8; KEY_SIZE];
    pbkdf2::<Hmac<Sha1>>(&password, salt, DEFAULT_ITER, &mut byte_key);

    let first = &blist[16..DEFAULT_PAGESIZE];
    let mac_salt: Vec<u8> = salt.iter().map(|b| b ^ 58).collect();
    let mut mac_key = vec![0u8; KEY_SIZE];
    pbkdf2::<Hmac<Sha1>>(&byte_key, &mac_salt, 2, &mut mac_key);

    let mut hash_mac = Hmac::<Sha1>::new_varkey(&mac_key).map_err(|_| "HMAC error")?;
    hash_mac.update(&first[..first.len() - 32]);
    hash_mac.update(&[1, 0, 0, 0]);

    if hash_mac.finalize().into_bytes() != first[first.len() - 32..first.len() - 12] {
        return Err(format!("[-] Password Error! (key:'{}'; db_path:'{}'; out_path:'{}')", key, db_path, out_path));
    }

    let new_blist: Vec<&[u8]> = blist[DEFAULT_PAGESIZE..].chunks_exact(DEFAULT_PAGESIZE).collect();

    let mut de_file = fs::File::create(out_path).map_err(|e| e.to_string())?;
    de_file.write_all(SQLITE_FILE_HEADER.as_bytes()).map_err(|e| e.to_string())?;

    let iv_first = &first[first.len() - 48..first.len() - 32];
    let cipher = Cbc::<Aes128>::new_var(&byte_key, iv_first).map_err(|e| e.to_string())?;
    let decrypted = cipher.decrypt_vec(&first[..first.len() - 48]).map_err(|_| "Decryption failed")?;

    de_file.write_all(&decrypted).map_err(|e| e.to_string())?;
    de_file.write_all(&first[first.len() - 48..]).map_err(|e| e.to_string())?;

    for i in new_blist {
        let iv = &i[i.len() - 48..i.len() - 32];
        let cipher = Cbc::<Aes128>::new_var(&byte_key, iv).map_err(|e| e.to_string())?;
        let decrypted = cipher.decrypt_vec(&i[..i.len() - 48]).map_err(|_| "Decryption failed")?;

        de_file.write_all(&decrypted).map_err(|e| e.to_string())?;
        de_file.write_all(&i[i.len() - 48..]).map_err(|e| e.to_string())?;
    }
    Ok(vec![true, db_path.to_string(), out_path.to_string(), key.to_string()])
}

fn batch_decrypt(key: &str, db_path: &str, out_path: &str) -> Result<Vec<Result<Vec<bool>, String>>, String> {
    if key.len() != 64 || !Path::new(out_path).exists() {
        return Err(format!("[-] (key:'{}' or out_path:'{}') Error!", key, out_path));
    }

    let mut process_list = Vec::new();

    if Path::new(db_path).is_file() {
        let inpath = db_path.to_string();
        let outpath = format!("{}/de_{}", out_path, Path::new(db_path).file_name().unwrap().to_str().unwrap());
        process_list.push((key, inpath, outpath));
    }
    else if Path::new(db_path).is_dir() {
        for entry in fs::read_dir(db_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let inpath = entry.path();
            if inpath.is_file() {
                let outpath = format!("{}/de_{}", out_path, inpath.file_name().unwrap().to_str().unwrap());
                process_list.push((key, inpath.to_str().unwrap().to_string(), outpath));
            }
        }
    }
    else {
        return Err(format!("[-] db_path:'{}' Error ", db_path));
    }

    let mut result = Vec::new();
    for (k, db, out) in process_list {
        result.push(decrypt(k, &db, &out));
    }

    // Remove empty directories
    for entry in fs::read_dir(out_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            if fs::read_dir(entry.path()).map_err(|e| e.to_string())?.count() == 0 {
                fs::remove_dir(entry.path()).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(result)
}
