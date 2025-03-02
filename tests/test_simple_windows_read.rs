#![cfg(target_os = "windows")]

use sysinfo::Pid;
use rs_class::win_ops::{SystemProcess, WinProcess};
use rs_class::typing::{DataType, IntSize, IntegerDataType};

#[test]
fn test_windows_simple_read() {
    let x: u64 = 0xDEADBEEF;
    let pid = Pid::from_u32(std::process::id());
    let mut process = WinProcess::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process.read_memory(&x as *const u64 as u64, size_of::<u64>() as u64).expect("Could not read memory");
    assert_eq!(read_mem.len(), size_of::<u64>(), "length should be equal");
    assert_eq!(read_mem[0], 0xEFu8);
}

#[test]
fn test_windows_read_with_default_type() {
    let x: u32 = 0xDEADBEEF;
    let pid = Pid::from_u32(std::process::id());
    let mut process = WinProcess::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process.read_memory(&x as *const u32 as u64, size_of::<u32>() as u64).expect("Could not read memory");
    assert_eq!(read_mem.len(), size_of::<u32>(), "length should be equal");
    let t = IntegerDataType::default();
    assert_eq!(t.from_bytes(&read_mem).expect("Could not convert to bytes"), "3735928559", "Values should be the same")
}

#[test]
fn test_windows_read_with_custom_type() {
    let x: u64 = 0xDEADBEEF;
    let pid = Pid::from_u32(std::process::id());
    let mut process = WinProcess::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process.read_memory(&x as *const u64 as u64, size_of::<u64>() as u64).expect("Could not read memory");
    assert_eq!(read_mem.len(), size_of::<u64>(), "length should be equal");
    let t = IntegerDataType::default().with_hex(true).with_signed(false).with_size(IntSize::Integer64);
    assert_eq!(t.from_bytes(&read_mem).expect("Could not convert to bytes"), "0xDEADBEEF", "Values should be the same")
}