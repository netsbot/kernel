#![no_std]
#![no_main]
extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;

use acpi::platform::InterruptModel;
use kernel::*;

bootloader_api::entry_point!(kernel_start, config = &kernel::BOOTLOADER_CONFIG);
#[unsafe(no_mangle)]
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    io::framebuffer::init(boot_info.framebuffer.as_mut().unwrap());
    io::serial_monitor::init(kernel::SERIAL_MONITOR_PORT);
    mem::init(&boot_info.memory_regions);
    gdt::init();
    idt::init();

    unsafe {
        io::acpi::init(
            boot_info
                .rsdp_addr
                .into_option()
                .expect("missing rsdp addr"),
        );

        if let InterruptModel::Apic(apic) = &io::acpi::ACPI_PLATFORM.get_unchecked().interrupt_model
        {
            io::apic::init(apic)
        } else {
            panic!("no xapic interrupt controller found")
        };
    }

    x86_64::instructions::interrupts::enable();

    println!("okay!");

    let mut exec = SimpleExecutor::new();

    exec.spawn(Task::new(io::keyboard::print_keypresses()));
    exec.run();

    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");

    hlt_loop()
}

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

use alloc::collections::VecDeque;
use core::pin::Pin;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
}

use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

impl SimpleExecutor {
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}
