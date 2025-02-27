use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::Foundation::{HANDLE, BOOL};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};

use sysinfo::Pid;

#[derive(Debug, PartialEq, Default)]
enum State {
    #[default]
    Created,
    Open(HANDLE),
    Closed,
}
impl State {
    fn handle(&self) -> Option<HANDLE> {
        match self {
            State::Open(handle) => Some(*handle),
            _ => None
        }
    }
}

#[derive(Debug)]
struct WinProcess {
    pid: Pid,
    state: State
}

impl WinProcess {
    pub fn new(pid: Pid) -> Self{
        Self {pid, state: State::Created}
    }

    pub fn open(&mut self) -> bool {
        match self.state {
            State::Created => {
                let handle = unsafe {
                    OpenProcess(PROCESS_VM_OPERATION|PROCESS_VM_READ|PROCESS_VM_WRITE, false.into(), self.pid.as_u32())
                };
                self.state = State::Open(handle);
                true
            },
            _ => false
        }
    }

    pub fn get_memory(&mut self, from: u64, size: usize) -> Result<Vec<u8>, String> {
        let handle = self.state.handle().ok_or("handle is closed or not yet opened")?;
        let mut read_buffer: Vec<u8> = Vec::with_capacity(size);
        let mut bytes_read = 0usize;
        unsafe {
            ReadProcessMemory(handle, from as *const core::ffi::c_void, read_buffer.as_mut_ptr() as *mut core::ffi::c_void, size.into(), &mut bytes_read);
        }
        return Ok(read_buffer);
    }
}