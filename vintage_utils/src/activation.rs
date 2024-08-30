use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

pub struct Activation {
    active: AtomicBool,
    notify: Notify,
}

impl Activation {
    pub fn new(active: bool) -> Self {
        Activation {
            active: AtomicBool::new(active),
            notify: Notify::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn set_active(&self, active: bool) {
        if active {
            let old_active = self.active.swap(true, Ordering::AcqRel);
            if !old_active {
                self.notify.notify_waiters();
            }
        } else {
            self.active.store(false, Ordering::Release);
        }
    }

    pub fn swap_active(&self, active: bool) -> bool {
        let old_active = self.active.swap(true, Ordering::AcqRel);
        if active && !old_active {
            self.notify.notify_waiters();
        }
        old_active
    }

    pub async fn wait(&self) {
        loop {
            let notified = self.notify.notified();
            if self.is_active() {
                break;
            }
            notified.await;
        }
    }
}
