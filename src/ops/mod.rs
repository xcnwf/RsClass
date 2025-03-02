use sysinfo::Pid;
use crate::typing::DataType;

#[cfg(target_os="windows")]
mod win;
#[cfg(target_os="windows")]
pub use win::WinProcess as Process;

pub trait SystemProcess {
    fn open(&mut self) -> Result<(), String>;
    fn pid(&self) -> Pid;
    fn read_memory(&mut self, location: u64, dt: &impl DataType) -> Result<Vec<u8>, String>;
    fn write_memory(&mut self, location: u64, what: Vec<u8>) -> Result<(), String>;
    fn close(&mut self);
}

