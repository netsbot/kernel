#![no_std]
#![no_main]

extern crate alloc;

use kernel::{tasks::executor::ASYNC_EXECUTOR, *};
use limine::{
    BaseRevision,
    request::{RequestsEndMarker, RequestsStartMarker},
};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    drivers::init_stdout();
    mem::init();
    arch::init();
    tasks::executor::init();
    drivers::init();

    unsafe { ASYNC_EXECUTOR.get_unchecked().lock().run() }

    println!("hello, world!");

    hlt_loop();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    hlt_loop()
}
