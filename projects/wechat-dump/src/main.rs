use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::str;

fn main() {
    let mut wechat_processes = find_wechat_processes();
    println!("Found {} WeChat processes", wechat_processes.len());
    for process in wechat_processes {
        println!("Process ID: {}", process);
    }
}

fn find_wechat_processes() -> HashSet<u32> {
    let mut wechat_processes = HashSet::new();
    let output = Command::new("tasklist")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to execute tasklist command");
    let output_str = str::from_utf8(&output.stdout).expect("Failed to convert output to string");
    for line in output_str.lines() {
        if line.contains("WeChat.exe") {
            let process_id = line
                .split_whitespace()
                .nth(1)
                .and_then(|id| id.parse::<u32>().ok())
                .unwrap_or(0);
            if process_id > 0 {
                wechat_processes.insert(process_id);
            }
        }
    }
    wechat_processes
}