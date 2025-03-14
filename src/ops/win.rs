use sysinfo::Pid;

use crate::typing::DataType;

use windows_sys::Win32::Foundation::{GetLastError, HANDLE};
use windows_sys::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

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
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct WinProcess {
    pid: Pid,
    state: State,
}

impl WinProcess {
    pub fn new(pid: Pid) -> Self {
        Self {
            pid,
            state: State::Created,
        }
    }
}
impl super::SystemProcess for WinProcess {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn open(&mut self) -> Result<(), String> {
        match self.state {
            State::Created => {
                let handle = unsafe {
                    OpenProcess(
                        PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
                        true.into(),
                        self.pid.as_u32(),
                    )
                };
                if handle.is_null() {
                    return Err(format!(
                        "Could not get an handle on the process, error : {}",
                        unsafe { GetLastError() }
                    ));
                }
                self.state = State::Open(handle);
                Ok(())
            }
            _ => Err("This process has already been opened.".into()),
        }
    }

    fn read_memory(&mut self, location: u64, dt: &impl DataType) -> Result<Vec<u8>, String> {
        let handle = self
            .state
            .handle()
            .ok_or("Handle is closed or not yet opened.")?;
        let size = dt.get_size();
        let mut read_buffer: Vec<u8> = Vec::with_capacity(size);
        let mut bytes_read = 0usize;
        unsafe {
            let r = ReadProcessMemory(
                handle,
                location as *const std::ffi::c_void,
                read_buffer.as_mut_ptr().cast::<core::ffi::c_void>(),
                size,
                &mut bytes_read,
            );
            if r == 0 {
                return Err(format!("Could not read memory, error: {}.", GetLastError()));
            }
            if bytes_read != size {
                return Err("Memory read has a smaller size than requested.".to_string());
            }
            read_buffer.set_len(bytes_read);
        };
        Ok(read_buffer)
    }

    fn write_memory(&mut self, location: u64, what: Vec<u8>) -> Result<(), String> {
        let handle = self
            .state
            .handle()
            .ok_or("Handle is closed or not yet opened.")?;
        let size = what.len();
        let mut bytes_written = 0usize;
        unsafe {
            let r = WriteProcessMemory(
                handle,
                location as *const std::ffi::c_void,
                what.as_ptr().cast::<core::ffi::c_void>(),
                size,
                &mut bytes_written,
            );
            if r == 0 {
                return Err(format!(
                    "Could not write memory, error: {}.",
                    GetLastError()
                ));
            }
            if bytes_written != size {
                return Err("Memory written was smaller than requested !".to_string());
            }
        };
        Ok(())
    }

    fn close(&mut self) {
        self.state = State::Closed;
    }
}
