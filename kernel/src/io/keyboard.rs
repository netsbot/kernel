use core::{
    pin::Pin,
    task::{Context, Poll},
};
use core::fmt::Debug;
use acpi::aml::object::Object::Event;
use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, StreamExt, task::AtomicWaker};
use pc_keyboard::{KeyEvent, ScancodeSet};
use spin::Once;

use crate::{print, println, warning};

const SCANCODE_QUEUE_LEN: usize = 128;

static WAKER: AtomicWaker = AtomicWaker::new();
static SCANCODE_QUEUE: Once<ArrayQueue<u8>> = Once::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        if queue.push(scancode).is_err() {
            warning!("scancode queue is full, dropping keyboard input")
        } else {
            WAKER.wake()
        }
    } else {
        warning!("scancode queue uninitialized")
    }
}

struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.call_once(|| ArrayQueue::new(SCANCODE_QUEUE_LEN));
        Self { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.get().expect("scancode queue uninitialized");

        // fast path

        if let Some(item) = queue.pop() {
            return Poll::Ready(Some(item));
        };

        WAKER.register(cx.waker());

        match queue.pop() {
            Some(item) => {
                WAKER.take();
                Poll::Ready(Some(item))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut stream = ScancodeStream::new();
    let mut kb = pc_keyboard::ScancodeSet1::new();

    while let Some(scancode) = stream.next().await {
        if let Ok(Some(event)) = kb.advance_state(scancode) {
            if let pc_keyboard::KeyState::Down = event.state {
                print!("{:?}", event.code)
            }
        }
    }
}
