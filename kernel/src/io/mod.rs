use core::fmt;

pub mod acpi;
pub mod apic;
pub mod framebuffer;
pub mod serial_monitor;
pub mod keyboard;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
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
