use crate::{dbus_queue::DBusQueue, infallible_property::InfalliblePropertyGetAndSubscribe};
use mini_sansio_dbus::{
    IncomingMessage, messages::network_manager::PrimaryDevice as PrimaryDeviceProperty,
};

pub struct PrimaryDevice {
    inner: InfalliblePropertyGetAndSubscribe<PrimaryDeviceProperty<String>>,
}

#[derive(Debug)]
pub enum PrimaryDeviceEvent {
    Connected(String),
    Disconnected,
}
impl From<&str> for PrimaryDeviceEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(String::from(path))
        }
    }
}

impl PrimaryDevice {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, path: String, q: &mut DBusQueue) {
        self.inner
            .get_and_subscribe(PrimaryDeviceProperty::new(path), q);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.inner.unsubscribe(q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<PrimaryDeviceEvent> {
        let path = self.inner.handle_reply_or_signal(message, q)?;
        Some(PrimaryDeviceEvent::from(path))
    }
}
