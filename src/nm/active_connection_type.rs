use crate::{dbus_queue::DBusQueue, infallible_property::InfalliblePropertyGetAndSubscribe};
use mini_sansio_dbus::{
    IncomingMessage,
    messages::network_manager::ActiveConnectionType as ActiveConnectionTypeProperty,
};

pub struct ActiveConnectionType {
    inner: InfalliblePropertyGetAndSubscribe<ActiveConnectionTypeProperty<String>>,
    path: Option<String>,
}

impl ActiveConnectionType {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(),
            path: None,
        }
    }

    pub(crate) fn start(&mut self, path: String, q: &mut DBusQueue) {
        self.inner
            .get_and_subscribe(ActiveConnectionTypeProperty::new(path.clone()), q);
        self.path = Some(path);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.inner.unsubscribe(q);
        self.path = None;
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<(bool, String)> {
        let type_ = self.inner.handle_reply_or_signal(message, q)?;
        let is_wireless = type_.contains("wireless");
        let path = self.path.as_ref()?.clone();
        Some((is_wireless, path))
    }
}
