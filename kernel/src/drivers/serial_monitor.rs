use core::fmt;

use spin::{Mutex, Once};
use x86_64::instructions::port::Port;

pub static WRITER: Once<Mutex<SerialMonitorWriter>> = Once::new();

pub fn init(port: u16) {
    WRITER.call_once(|| Mutex::new(SerialMonitorWriter::new(port)));
}

#[derive(Debug)]
pub struct SerialMonitorWriter {
    port: Port<u8>,
}

impl SerialMonitorWriter {
    pub fn new(port: u16) -> Self {
        Self {
            port: Port::new(port),
        }
    }
}

impl fmt::Write for SerialMonitorWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for char in s.chars() {
            unsafe { self.port.write(char as u8) }
        }

        Ok(())
    }
}
