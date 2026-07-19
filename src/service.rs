use crate::{dbus::DBus, dbus_queue::DBusQueue, nm::NM};
use anyhow::Result;
use nm_mon::Event;
use pollsendd::Service;
use rustix::event::{PollFd, PollFlags};
use std::os::fd::AsRawFd;

pub struct NmMonService {
    queue: DBusQueue,
    nm: NM,
    dbus: DBus,
    readbuf: Vec<u8>,
    last_ssid_and_strength_event: Option<Event>,
}

impl NmMonService {
    pub(crate) fn new() -> Result<Self> {
        let mut queue = DBusQueue::new()?;
        let nm = NM::new(&mut queue);

        let dbus = DBus::new()?;
        let readbuf = vec![0; 10 * 1_024];

        Ok(Self {
            queue,
            nm,
            dbus,
            readbuf,
            last_ssid_and_strength_event: None,
        })
    }
}

impl Service<1> for NmMonService {
    type Error = NmMonServiceError;

    fn pollfds(&mut self) -> Result<impl Iterator<Item = PollFd<'_>>, Self::Error> {
        Ok(std::iter::once(
            self.dbus.as_pollfd(&mut self.readbuf, &self.queue)?,
        ))
    }

    fn owns_fd(&self, fd: i32) -> bool {
        self.dbus.as_raw_fd() == fd
    }

    fn on_readable(
        &mut self,
        _fd: i32,
        mut broadcast: impl FnMut(&[u8]),
    ) -> Result<(), Self::Error> {
        if let Some(message) = self.dbus.read(&mut self.readbuf, &self.queue)? {
            let mut events = vec![];
            self.nm.handle(message, &mut events, &mut self.queue);
            log::trace!("{events:?}");
            for event in events {
                let buf = event.serialize();
                (broadcast)(&buf);
                if matches!(event, Event::SsidAndStrength { .. }) {
                    self.last_ssid_and_strength_event = Some(event);
                }
            }
        }

        Ok(())
    }

    fn on_writable(&mut self, _fd: i32, _broadcast: impl FnMut(&[u8])) -> Result<(), Self::Error> {
        self.dbus.write(&mut self.readbuf, &mut self.queue)?;
        Ok(())
    }

    fn on_poll_error(&mut self, _fd: i32, revents: PollFlags) -> Result<(), Self::Error> {
        Err(anyhow::anyhow!("DBus poll error: got revents {revents:?}").into())
    }

    fn on_client_connected(&mut self, mut write: impl FnMut(Vec<u8>)) -> Result<(), Self::Error> {
        if let Some(last_ssid_and_strength_event) = &self.last_ssid_and_strength_event {
            (write)(last_ssid_and_strength_event.serialize().to_vec());
        }
        Ok(())
    }

    fn on_client_request(&mut self, _request: [u8; 1]) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct NmMonServiceError(anyhow::Error);
impl std::fmt::Display for NmMonServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Debug for NmMonServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl core::error::Error for NmMonServiceError {}
impl From<anyhow::Error> for NmMonServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}
