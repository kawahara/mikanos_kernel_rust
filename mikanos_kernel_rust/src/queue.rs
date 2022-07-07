use crate::sync::once_cell::OnceCell;
use alloc::sync::Arc;
use crossbeam_queue::ArrayQueue;

#[derive(Debug)]
pub enum QueueEventType {
    InterruptXHCI,
}

#[derive(Debug)]
pub struct QueueEvent {
    pub event_type: QueueEventType,
}

static EVENT_QUEUE: OnceCell<Arc<ArrayQueue<QueueEvent>>> = OnceCell::uninit();
const EVENT_QUEUE_SIZE: usize = 100;

pub fn init() {
    EVENT_QUEUE.init_once(|| Arc::new(ArrayQueue::<QueueEvent>::new(EVENT_QUEUE_SIZE)));
}

pub fn event_queue() -> &'static Arc<ArrayQueue<QueueEvent>> {
    EVENT_QUEUE.get()
}
