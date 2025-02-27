use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::Foundation::{HANDLE, GetLastError};
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

pub trait SystemProcess {
    fn open(&mut self) -> Result<(), String>;
    fn pid(&self) -> Pid;
    fn read_memory(&mut self, from: u64, size: u64) -> Result<Vec<u8>, String>;
    fn write_memory(&mut self, from: u64, what: Vec<u8>) -> Result<(), String>;
}

#[derive(Debug)]
pub struct WinProcess {
    pid: Pid,
    state: State
}

impl WinProcess {
    pub fn new(pid: Pid) -> Self{
        Self {pid, state: State::Created}
    }
}
impl SystemProcess for WinProcess {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn open(&mut self) -> Result<(),String> {
        match self.state {
            State::Created => {
                let handle = unsafe {
                    OpenProcess(PROCESS_VM_OPERATION|PROCESS_VM_READ|PROCESS_VM_WRITE, true.into(), self.pid.as_u32())
                };
                if handle.is_null() {
                    return Err(format!("Could not get an handle on the process, error : {}", unsafe { GetLastError() }))
                }
                self.state = State::Open(handle);
                Ok(())
            },
            _ => Err("This process has already been opened.".into())
        }
    }

    fn read_memory(&mut self, from: u64, size: u64) -> Result<Vec<u8>, String> {
        let handle = self.state.handle().ok_or("Handle is closed or not yet opened.")?;
        let mut read_buffer: Vec<u8> = Vec::with_capacity(size as usize);
        let mut bytes_read = 0usize;
        unsafe {
            let r = ReadProcessMemory(handle, from as *const core::ffi::c_void, read_buffer.as_mut_ptr() as *mut core::ffi::c_void, size as usize, &mut bytes_read);
            if r == 0 {
                return Err(format!("Could not read memory, error: {}.",GetLastError()))
            }
            if bytes_read as u64 != size {
                return Err(format!("Memory read has a smaller size than requested."))
            }
            read_buffer.set_len(bytes_read);
        };
        return Ok(read_buffer);
    }
    fn write_memory(&mut self, from: u64, what: Vec<u8>) -> Result<(), String> {
        todo!("winprocess: write_memory")
    }
}