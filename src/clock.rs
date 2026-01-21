extern crate alloc;

use core::cell::RefCell;

use alloc::rc::Rc;
use embassy_time::Instant;

#[derive(Clone)]
pub struct Clock {
    timezone: jiff::tz::TimeZone,
    offset: Rc<RefCell<u64>>,
}

impl Clock {
    pub fn new(timezone: jiff::tz::TimeZone) -> Self {
        Self {
            timezone,
            offset: Rc::new(RefCell::new(0)),
        }
    }

    /// Sync local clock with `ntp_sec` given in seconds since UNIX epoch.
    pub fn sync(&self, ntp_sec: u64) {
        let now = Instant::now().as_secs();
        let offset = ntp_sec.saturating_sub(now);
        *self.offset.borrow_mut() = offset;
    }

    /// Return [`jiff::Zoned`] for the current date and time.
    pub fn now(&self) -> jiff::Zoned {
        let now = Instant::now().as_secs() + *self.offset.borrow();

        jiff::Timestamp::from_second(now as i64)
            .expect("creating a jiff timestamp")
            .to_zoned(self.timezone.clone())
    }
}
