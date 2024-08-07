use std::collections::HashSet;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use std::process::Command;
use std::process::Stdio;
use std::io::Read;
use winapi::um::winnt::{PROCESS_VM_OPERATION, PROCESS_VM_READ};
use winapi::um::memoryapi::{ReadProcessMemory};
use winapi::shared::minwindef::{DWORD, LPVOID, BOOL};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::processthreadsapi::{OpenProcess};
use winapi::um::winnt::{HANDLE, PROCESS_VM_OPERATION, PROCESS_VM_READ};
use std::mem;


pub fn find_wechat_processes() -> HashSet<u32> {
    let mut wechat_processes = HashSet::new();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return wechat_processes;
        }

        let mut pe32: PROCESSENTRY32 = std::mem::zeroed();
        pe32.dwSize = size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut pe32) == 0 {
            CloseHandle(snapshot);
            return wechat_processes;
        }

        loop {
            if pe32.szExeFile.iter().take_while(|&c| *c != 0).map(|&c| c as u8 as char).collect::<String>() == "WeChat.exe" {
                wechat_processes.insert(pe32.th32ProcessID);
            }

            if Process32Next(snapshot, &mut pe32) == 0 {
                break;
            }
        }

        CloseHandle(snapshot);
    }

    wechat_processes
}

fn read_process_memory(process_id: u32, start_offset: usize, end_offset: usize) -> Result<Vec<u8>, String> {
    let mut buffer = vec![0u8; end_offset - start_offset];

    unsafe {
        let process_handle = OpenProcess(PROCESS_VM_OPERATION | PROCESS_VM_READ, 0, process_id);
        if process_handle.is_null() {
            return Err(format!("Failed to open process: {}", GetLastError()));
        }

        let mut bytes_read: usize = 0;
        let success = ReadProcessMemory(
            process_handle,
            start_offset as *const _,
            buffer.as_mut_ptr() as *mut _,
            buffer.len(),
            &mut bytes_read,
        );

        if success == 0 {
            let error_code = GetLastError();
            CloseHandle(process_handle);
            return Err(format!("Failed to read process memory: {}", error_code));
        }

        CloseHandle(process_handle);
        Ok(buffer)
    }
}

fn main() {
    for process_id in find_wechat_processes() {
        let start_address = 0x0;
        let end_address = 0x2000;
        let memory = read_process_memory(process_id, start_address, end_address);
        println!("Memory contents: {:?}", memory);
    }
}