use core::fmt;
use bootloader_api::info::{FrameBuffer, Optional};
use crate::SERIAL_MONITOR_PORT;
use crate::drivers::keyboard::print_keypresses;
use crate::tasks::executor::{Task, ASYNC_EXECUTOR};

mod framebuffer;
mod serial_monitor;
pub(crate) mod keyboard;

pub fn init_stdout(framebuffer: &'static mut Optional<FrameBuffer>) {
    framebuffer::init(framebuffer.as_mut().unwrap());
    serial_monitor::init(SERIAL_MONITOR_PORT);
}

pub fn init() {
    let mut executor = unsafe { ASYNC_EXECUTOR.get_unchecked() }.lock();
    executor.spawn(Task::new(print_keypresses()));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! warning {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("WARNING: {}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        framebuffer::WRITER
            .get()
            .expect("framebuffer not initialized")
            .lock()
            .write_fmt(args)
            .unwrap();

        serial_monitor::WRITER
            .get()
            .expect("serial monitor not initialized")
            .lock()
            .write_fmt(args)
            .unwrap();
    })
}
