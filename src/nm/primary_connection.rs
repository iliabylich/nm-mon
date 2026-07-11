use crate::{dbus_queue::DBusQueue, infallible_property::InfalliblePropertyGetAndSubscribe};
use mini_sansio_dbus::{
    IncomingMessage, messages::network_manager::PrimaryConnection as PrimaryConnectionProperty,
};

pub struct PrimaryConnection {
    inner: InfalliblePropertyGetAndSubscribe<PrimaryConnectionProperty>,
}

pub enum PrimaryConnectionEvent {
    Connected(String),
    Disconnected,
}

impl From<&str> for PrimaryConnectionEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(String::from(path))
        }
    }
}

impl PrimaryConnection {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, q: &mut DBusQueue) {
        self.inner.get_and_subscribe(PrimaryConnectionProperty, q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<PrimaryConnectionEvent> {
        let path = self.inner.handle_reply_or_signal(message, q)?;
        Some(PrimaryConnectionEvent::from(path))
    }
}
