

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr,
};
use windows::{
    core::*,
    Win32::{Foundation::*, System::Diagnostics::Debug::ReadProcessMemory},
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

fn read_memory_string(h_process: HANDLE, address: usize, n_size: usize) -> Result<String> {
    let mut buffer: Vec<u8> = vec![0; n_size];
    unsafe { ReadProcessMemory(h_process, address as _, buffer.as_mut_ptr() as _, n_size as _, None)? };
    let text = buffer.iter().take_while(|&&b| b != 0).copied().collect::<Vec<u8>>();
    Ok(String::from_utf8_lossy(&text).trim().to_string())
}

fn read_memory_address(h_process: HANDLE, address: usize, address_len: usize) -> Result<usize> {
    let mut buffer: Vec<u8> = vec![0; address_len];

    unsafe { ReadProcessMemory(h_process, address as _, buffer.as_mut_ptr() as _, address_len as _, None)? };

    Ok(usize::from_le_bytes(buffer.try_into().unwrap()))
}

fn read_key(h_process: HANDLE, address: usize) -> Result<String> {
    let address_len = 8;
    if let Some(key_address) = read_memory_address(h_process, address, address_len) {
        let mut key_buffer: Vec<u8> = vec![0; 32];
        unsafe { ReadProcessMemory(h_process, key_address as _, key_buffer.as_mut_ptr() as _, 32, None)? }
        return Ok(hex::encode(key_buffer));
    }
    Ok("None".to_string())
}

fn read_info(version_list: &HashMap<String, Vec<usize>>) -> Result<Vec<HashMap<String, String>>, String> {
    let mut wechat_process = Vec::new();
    let mut result = Vec::new();

    // Find WeChat process
    for process in psutil::process_iter() {
        if process.name() == "WeChat.exe" {
            wechat_process.push(process);
        }
    }

    if wechat_process.is_empty() {
        return Err("[-] WeChat No Run".to_string());
    }

    for process in wechat_process {
        let mut tmp_rd = HashMap::new();
        tmp_rd.insert("pid".to_string(), process.pid.to_string());

        let version = process.version()?;
        tmp_rd.insert("version".to_string(), version.clone());

        let bias_list = version_list.get(&version).ok_or(format!("[-] WeChat Current Version {} Is Not Supported", version))?;

        let wechat_base_address = process
            .memory_maps()?
            .iter()
            .find(|m| m.path.contains("WeChatWin.dll"))
            .ok_or("[-] WeChat WeChatWin.dll Not Found".to_string())?
            .addr;

        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process.pid) };

        let account_address = wechat_base_address + bias_list[1];
        tmp_rd.insert("account".to_string(), read_memory_string(handle, account_address, 32));

        result.push(tmp_rd);
    }

    Ok(result)
}

fn main() {
    let list = HashMap::new();
    for x in read_info(&list).unwrap() {
        for (key, value) in x {
            println!("{key}: {value}")
        }
    }
}
