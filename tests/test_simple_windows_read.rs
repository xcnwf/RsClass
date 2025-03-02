#![cfg(target_os = "windows")]

use sysinfo::Pid;
use rs_class::win_ops::{SystemProcess, WinProcess};
use rs_class::typing::{DataType, IntSize, IntegerDataType};

#[test]
fn test_windows_read_with_default_type() {
    let x: u32 = 0xDEADBEEF;
    let dt = IntegerDataType::default();

    let pid = Pid::from_u32(std::process::id());
    let mut process = WinProcess::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process.read_memory(&raw const x as u64, &dt).expect("Could not read memory");
    assert_eq!(read_mem.len(), size_of::<u32>(), "length should be equal");
    assert_eq!(dt.from_bytes(&read_mem).expect("Could not convert to bytes"), "3735928559", "Values should be the same")
}

#[test]
fn test_windows_read_with_custom_type() {
    let x: u64 = 0xDEADBEEF;
    let dt = IntegerDataType::default().with_hex(true).with_signed(false).with_size(IntSize::Integer64);

    let pid = Pid::from_u32(std::process::id());
    let mut process = WinProcess::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process.read_memory(&raw const x as u64, &dt).expect("Could not read memory");
    assert_eq!(read_mem.len(), size_of::<u64>(), "length should be equal");
    assert_eq!(dt.from_bytes(&read_mem).expect("Could not convert to bytes"), "0xDEADBEEF", "Values should be the same")
}