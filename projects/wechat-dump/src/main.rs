#![feature(strict_provenance)]

use proc_mem::{Module, ProcMemError, Process};
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    ffi::{CStr, CString},
    fmt::Formatter,
    ptr,
};
use windows::{
    core::*,
    System::Diagnostics::ProcessDiagnosticInfo,
    Wdk::System::Threading::{NtQueryInformationProcess, PROCESSINFOCLASS},
    Win32::{
        Foundation::*,
        Storage::FileSystem::VS_FIXEDFILEINFO,
        System::{
            Diagnostics::{
                Debug::ReadProcessMemory,
                ToolHelp::{CreateToolhelp32Snapshot, Process32First, PROCESSENTRY32},
            },
            Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, PAGE_READWRITE, VIRTUAL_ALLOCATION_TYPE},
            ProcessStatus::{EnumProcessModules, EnumProcessModulesEx, LIST_MODULES_ALL},
            Threading::{
                GetProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_ALL_ACCESS, PROCESS_BASIC_INFORMATION,
                PROCESS_NAME_FORMAT,
            },
        },
    },
};

mod secrets;

pub use crate::secrets::{UserInfo, UserInfoOffset};

pub fn find_wechat_progresses() -> Result<Vec<ProcessDiagnosticInfo>> {
    let mut wechat_processes = Vec::with_capacity(1);

    // 获取所有正在运行的进程
    let process_infos = ProcessDiagnosticInfo::GetForProcesses()?;

    // 遍历进程
    for process_info in process_infos {
        let process_name = process_info.ExecutableFileName()?;
        if process_name == "WeChat.exe" {
            wechat_processes.push(process_info);
        }
    }

    return Ok(wechat_processes);
}

fn find_base_address(handle: HANDLE) -> Result<usize> {
    let mut process_info = PROCESS_BASIC_INFORMATION::default();
    let nt_status = unsafe {
        NtQueryInformationProcess(
            handle,
            PROCESSINFOCLASS::default(),
            &mut process_info as *mut _ as *mut core::ffi::c_void,
            size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            std::ptr::null_mut(),
        )
    };
    if nt_status.is_ok() { Ok(process_info.PebBaseAddress as usize) } else { Err(windows::core::Error::from(nt_status)) }
}

fn read_memory_string(h_process: HANDLE, address: usize, n_size: usize) -> Result<String> {
    let mut buffer: Vec<u8> = vec![0; n_size];
    unsafe { ReadProcessMemory(h_process, address as _, buffer.as_mut_ptr() as _, n_size as _, None)? };
    let text = buffer.iter().take_while(|&&b| b != 0).copied().collect::<Vec<u8>>();
    Ok(String::from_utf8_lossy(&text).trim().to_string())
}

fn read_memory_address(h_process: HANDLE, offset: usize, length: usize) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = vec![0; length];
    unsafe { ReadProcessMemory(h_process, offset as _, buffer.as_mut_ptr() as _, length as _, None)? };
    Ok(buffer)
}

// fn read_key(h_process: HANDLE, address: usize) -> Result<String> {
//     let address_len = 8;
//     if let Ok(key_address) = read_memory_address(h_process, address, address_len) {
//         let mut key_buffer: Vec<u8> = vec![0; 32];
//         unsafe { ReadProcessMemory(h_process, key_address as _, key_buffer.as_mut_ptr() as _, 32, None)? }
//         return Ok(hex::encode(key_buffer));
//     }
//     Ok("None".to_string())
// }

fn read_info(version_list: &HashMap<String, Vec<usize>>) -> Result<HashMap<String, String>> {
    for process in find_wechat_progresses()? {
        let mut tmp_rd = HashMap::new();
        tmp_rd.insert("pid".to_string(), process.ProcessId()?.to_string());

        // let version = process.ExecutableFileName()?;
        // tmp_rd.insert("version".to_string(), version.clone());
        //
        // let bias_list = version_list.get(&version).ok_or(format!("[-] WeChat Current Version {} Is Not Supported", version))?;
        //

        // let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process.pid) };
        //
        // let account_address = wechat_base_address + bias_list[1];
        // tmp_rd.insert("account".to_string(), read_memory_string(handle, account_address, 32));
        //
        // result.push(tmp_rd);
        return Ok(tmp_rd);
    }

    Ok(Default::default())
}

pub struct WeChatProcess {
    process: Process,
    module: Module,
}



fn main() -> Result<()> {
    let mut wechats = Vec::with_capacity(1);
    let processes = Process::all_with_name("WeChat.exe")?;
    for wechat in processes {
        wechats.push(WeChatProcess { process: wechat, module: wechat.module("WeChatWin.dll")? });
    }

    Ok(())
}

fn get_process_version(handle: HANDLE) -> windows::core::Result<VS_FIXEDFILEINFO> {
    let mut buffer = Vec::with_capacity(MAX_PATH as usize);
    let mut size = buffer.capacity() as u32;
    unsafe {
        if QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT::default(), *buffer.as_mut_ptr(), &mut size).is_ok() {
            buffer.set_len(size as usize);
            let mut version_info_value = VS_FIXEDFILEINFO::default();
            if windows::Win32::Storage::FileSystem::GetFileVersionInfoW(
                PCWSTR::from_raw(buffer.as_ptr() as *const u16),
                0,
                std::mem::size_of::<windows::Win32::Storage::FileSystem::VS_FIXEDFILEINFO>() as u32,
                &mut version_info_value as *mut _ as *mut core::ffi::c_void,
            )
            .is_ok()
            {
                return Ok(version_info_value);
            }
            else {
                return Err(windows::core::Error::from_win32());
            }
        }
        else {
            return Err(windows::core::Error::from_win32());
        }
    }
}

fn find_wechat_module(process_handle: HANDLE) -> Result<HMODULE> {
    let mut needed = 0;
    let mut modules: Vec<HMODULE> = Vec::with_capacity(256);
    unsafe {
        EnumProcessModulesEx(
            process_handle,
            modules.as_mut_ptr(),
            (modules.capacity() * size_of::<HMODULE>()) as u32,
            &mut needed,
            LIST_MODULES_ALL,
        )?;
        modules.set_len(needed as usize / size_of::<HMODULE>());
        for module in modules {
            let mut name_buffer = [0u16; MAX_PATH as usize];
            let _ = windows::Win32::System::ProcessStatus::GetModuleBaseNameW(process_handle, module, &mut name_buffer);
            let dll = String::from_utf16_lossy(&name_buffer);
        }
    }
    Err(windows::core::Error::from_win32())
}

// fn K32GetModuleBaseNameW(hProcess: HANDLE, hModule: HMODULE, lpBaseName: *mut UNICODE_STRING) -> BOOL {
//     unsafe {
//         windows::Win32::System::Diagnostics::ToolHelp::K32GetModuleBaseNameW(hProcess, hModule, lpBaseName, MAX_PATH as u32)
//     }
// }

fn find_unicode_string(handle: HANDLE, address: usize, target: &str) -> Result<Vec<usize>> {
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    unsafe {
        let _ = VirtualQueryEx(handle, None, &mut mbi, size_of::<MEMORY_BASIC_INFORMATION>());
        let mut buffer: Vec<u8> = vec![0; mbi.RegionSize];
        println!("占用内存: {}", mbi.RegionSize);
        ReadProcessMemory(handle, address as *const _, buffer.as_mut_ptr() as *mut _, mbi.RegionSize, None)?;

        let mut offset = 0;
        while offset + target.len() < mbi.RegionSize {
            let slice = buffer.get_unchecked(offset..=offset + target.len());
            if slice.contains(&0) {
            }
            else {
                println!("{}", String::from_utf8_lossy(slice));
            }
            if slice.eq(target.as_bytes()) {
                return Ok(vec![offset]);
            }
            offset += 1;
        }
    }
    Ok(vec![])
}

unsafe fn all_memory_of_process(handle: HANDLE) -> Result<Vec<u8>> {
    let mut mbi: MEMORY_BASIC_INFORMATION = MEMORY_BASIC_INFORMATION::default();
    let mut all_memory: Vec<u8> = Vec::new();
    let mut address = 0;

    loop {
        let query = VirtualQueryEx(
            handle,
            Some(address as *const core::ffi::c_void),
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        );

        if query == 0 {
            break; // 查询失败，退出循环
        }
        println!("RegionSize: {}", mbi.RegionSize);
        let mut buffer: Vec<u8> = vec![0; mbi.RegionSize];
        match ReadProcessMemory(handle, address as *const _, buffer.as_mut_ptr() as *mut _, mbi.RegionSize, None) {
            Ok(o) => {}
            Err(e) => println!("ReadProcessMemory failed: {:?}", e),
        }
        all_memory.extend(buffer);
        address += mbi.RegionSize;
    }

    Ok(all_memory)
}
