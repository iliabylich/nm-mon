use crate::{dbus_queue::DBusQueue, infallible_property::InfalliblePropertyGetAndSubscribe};
use mini_sansio_dbus::{
    IncomingMessage, messages::network_manager::ActiveAccessPoint as ActiveAccessPointProperty,
};

pub struct ActiveAccessPoint {
    inner: InfalliblePropertyGetAndSubscribe<ActiveAccessPointProperty<String>>,
}

#[derive(Debug)]
pub enum ActiveAccessPointEvent {
    Connected(String),
    Disconnected,
}
impl From<&str> for ActiveAccessPointEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(String::from(path))
        }
    }
}

impl ActiveAccessPoint {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, path: String, q: &mut DBusQueue) {
        self.inner
            .get_and_subscribe(ActiveAccessPointProperty::new(path), q);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.inner.unsubscribe(q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<ActiveAccessPointEvent> {
        let path = self.inner.handle_reply_or_signal(message, q)?;
        Some(ActiveAccessPointEvent::from(path))
    }
}
