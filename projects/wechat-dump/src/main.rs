use std::{
    collections::{BTreeSet, HashMap},
    ffi::{CStr, CString},
    ptr,
};
use windows::{
    core::*,
    System::Diagnostics::ProcessDiagnosticInfo,
    Wdk::System::Threading::{NtQueryInformationProcess, PROCESSINFOCLASS},
    Win32::{
        Foundation::*,
        System::{
            Diagnostics::Debug::ReadProcessMemory,
            ProcessStatus::EnumProcessModules,
            Threading::{GetProcessId, OpenProcess, PROCESS_ALL_ACCESS, PROCESS_BASIC_INFORMATION},
        },
    },
};

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

fn main() -> Result<()> {
    for progress in find_wechat_progresses()? {
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, progress.ProcessId()?)? };
        let out = find_base_address(handle)?;
        println!("out: 0x{out:X}");
        let data = read_memory_address(handle, out + 100, 16)?;
        println!("data: {data:?}")
    }
    Ok(())
}

fn get_process_version(process: &windows::System::Diagnostics::ProcessDiagnosticInfo) -> windows::core::Result<String> {
    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process.ProcessId()?) };
    let mut buffer = Vec::with_capacity(MAX_PATH as usize);
    let mut size = buffer.capacity() as u32;

    let status = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT::ProcessNameDosPath,
            buffer.as_mut_ptr() as PWSTR,
            &mut size,
        )
    };

    if status != 0 {
        buffer.set_len(size as usize);
        let version_info = windows::Win32::System::LibraryLoader::GetFileVersionInfo(
            PCWSTR::from_raw(buffer.as_ptr()),
            0,
        );
        if version_info.is_ok() {
            let version_info_value = version_info?;
            let version_info_string = version_info_value.FileVersion.to_string();
            return Ok(version_info_string);
        }
    } else {
        let error = unsafe { GetLastError() };
        return Err(windows::core::Error::from(NTSTATUS(error as i32)));
    }

    Err(windows::core::Error::from_win32())
}