use crate::typing::DataType;
use sysinfo::Pid;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::WinProcess as Process;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinProcess as Process;

pub trait SystemProcess {
    fn open(&mut self) -> Result<(), String>;
    fn pid(&self) -> Pid;
    fn read_memory(&mut self, location: u64, dt: &impl DataType) -> Result<Vec<u8>, String>;
    fn write_memory(&mut self, location: u64, what: Vec<u8>) -> Result<(), String>;
    fn close(&mut self);
}
