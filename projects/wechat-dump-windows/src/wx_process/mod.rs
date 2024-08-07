use proc_mem::{Module, ProcMemError, Process};

pub struct WeChatProcess {
    pub(crate) process: Process,
    pub(crate) module: Module,
}

impl WeChatProcess {
    pub fn find() -> Result<Vec<Self>, ProcMemError> {
        let mut wechats = Vec::with_capacity(1);
        let processes = Process::all_with_name("WeChat.exe")?;
        for wechat in processes {
            let module = wechat.module("WeChatWin.dll")?;
            wechats.push(WeChatProcess { process: wechat, module });
        }
        return Ok(wechats);
    }
}
