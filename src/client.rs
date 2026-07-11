use anyhow::{Result, ensure};
use nm_mon::Event;
use rustix::{
    event::{PollFd, PollFlags},
    net::{RecvFlags, SendFlags},
};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd};

pub struct Client {
    fd: OwnedFd,
}

impl Client {
    pub(crate) const fn new(fd: OwnedFd) -> Self {
        Self { fd }
    }

    pub(crate) fn read(&self) -> Result<()> {
        log::info!("Reading 1 byte from client {}..", self.as_raw_fd());
        rustix::net::recv(&self.fd, &mut [0; 1], RecvFlags::empty())?;
        Ok(())
    }

    pub(crate) fn write(&self, event: &Event) -> Result<()> {
        log::info!("Sending {event:?} to client {}..", self.as_raw_fd());
        let buf = event.serialize();
        let bytes_written = rustix::net::send(&self.fd, &buf, SendFlags::NOSIGNAL)?;
        ensure!(bytes_written == buf.len());
        Ok(())
    }

    pub(crate) fn as_pollfd(&self) -> PollFd<'_> {
        PollFd::new(self, PollFlags::IN)
    }
}

impl AsFd for Client {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for Client {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}
