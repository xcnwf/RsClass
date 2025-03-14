#![cfg(target_os = "windows")]

use rs_class::ops::{Process, SystemProcess};
use rs_class::typing::{DataType, IntSize, IntegerDataType};
use sysinfo::Pid;

#[test]
fn test_windows_read_with_default_type() {
    let x: u32 = 0xDEAD_BEEF;
    let dt = IntegerDataType::default();

    let pid = Pid::from_u32(std::process::id());
    let mut process = Process::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process
        .read_memory(&raw const x as u64, &dt)
        .expect("Could not read memory");
    assert_eq!(read_mem.len(), dt.get_size(), "length should be equal");
    assert_eq!(
        dt.bytes_to_string(&read_mem)
            .expect("Could not convert to bytes"),
        "3735928559",
        "Values should be the same"
    );
}

#[test]
fn test_windows_read_with_custom_type() {
    let x: u64 = 0xDEAD_BEEF;
    let dt = IntegerDataType::default()
        .with_hex(true)
        .with_signed(false)
        .with_size(IntSize::Integer64);

    let pid = Pid::from_u32(std::process::id());
    let mut process = Process::new(pid);
    process.open().expect("Could not open the process");
    let read_mem = process
        .read_memory(&raw const x as u64, &dt)
        .expect("Could not read memory");
    assert_eq!(read_mem.len(), dt.get_size(), "length should be equal");
    assert_eq!(
        dt.bytes_to_string(&read_mem)
            .expect("Could not convert to bytes"),
        "0xDEADBEEF",
        "Values should be the same"
    );
}
