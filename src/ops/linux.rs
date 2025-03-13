use std::fs::File;
use std::io::{BufReader, SeekFrom};
use sysinfo::Pid;

use crate::typing::DataType;

#[derive(Debug, PartialEq, Default)]
enum State {
    #[default]
    Created,
    // Only bufreader is needed because read calls will be frequent, but not write calls, which should be immediate
    Open(BufReader<File>),
    Closed,
}
impl State {
    fn memfile(&self) -> Option<&HANDLE> {
        match self {
            State::Open(file) => Some(file),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct LinProcess {
    pid: Pid,
    state: State,
}

impl LinProcess {
    pub fn new(pid: Pid) -> Self {
        Self {
            pid,
            state: State::Created,
        }
    }
}
impl super::SystemProcess for LinProcess {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn open(&mut self) -> Result<(), String> {
        match self.state {
            State::Created => {
                let memfile = File::options()
                    .read(true)
                    .write(true)
                    .open("/proc/{}/mem")
                    .map_err(|err| err.to_string())?;
                self.state = State::Open(BufReader::new(memfile));
                Ok(())
            }
            _ => Err("This process has already been opened.".into()),
        }
    }

    fn read_memory(&mut self, location: u64, dt: &impl DataType) -> Result<Vec<u8>, String> {
        let mut memfile = self
            .state
            .memfile()
            .ok_or("Process is closed or not yet opened.")?;
        let size = dt.get_size();
        let mut read_buffer: Vec<u8> = Vec::with_capacity(size);

        // SAFETY:
        // 1. elements will get initialized by the read_exact if there are no errors (else the buffer is dropped)
        // 2. size <= size, set with with_capacity.
        // So set_len is safe to call
        unsafe {
            read_buffer.set_len(size);
        }

        if location != memfile.stream_position() {
            memfile.seek(SeekFrom::start(location));
        }

        memfile
            .read_exact(&mut read_buffer)
            .map_err(|err| err.to_string());

        return Ok(read_buffer);
    }

    fn write_memory(&mut self, location: u64, what: Vec<u8>) -> Result<(), String> {
        let mut memfile = self
            .state
            .memfile()
            .ok_or("Process is closed or not yet opened.")?;
        let size = what.len();
        let mut bytes_written = 0usize;

        if location != memfile.stream_position() {
            memfile.seek(SeekFrom::start(location));
        }

        let bytes_written = memfile
            .get_mut_ref()
            .write(&what)
            .map_err(|err| err.to_string())?;
        if bytes_written != size {
            return Err(format!("Memory written was smaller than requested !"));
        }

        return Ok(());
    }

    fn close(&mut self) {
        self.state = State::Closed;
    }
}
