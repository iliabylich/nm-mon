use crate::{args::Args, client::Client, nm::Event};
use anyhow::{Context as _, Result};
use rustix::event::{PollFd, PollFlags};
use std::{
    collections::HashMap,
    os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd},
};

pub struct Server {
    fd: OwnedFd,
    fd_to_client: HashMap<i32, Client>,
}

#[expect(dead_code)]
impl Server {
    pub(crate) fn new(args: &Args) -> Result<Self> {
        Ok(Self {
            fd: args.get_server_fd()?,
            fd_to_client: HashMap::new(),
        })
    }

    pub(crate) fn pollfds(&self) -> impl Iterator<Item = PollFd<'_>> {
        self.fd_to_client
            .values()
            .map(|client| client.as_pollfd())
            .chain([PollFd::new(&self.fd, PollFlags::IN)])
    }

    pub(crate) fn accept(&mut self) -> Result<()> {
        log::trace!("Accepting a new client");
        let fd = rustix::net::accept(&self.fd).context("failed to accept()")?;
        self.fd_to_client.insert(fd.as_raw_fd(), Client::new(fd));
        Ok(())
    }

    pub(crate) fn broadcast(&mut self, event: &Event) {
        log::info!("Sending {event:?} to {} clients..", self.fd_to_client.len());

        let mut fds_to_drop = vec![];

        for (fd, client) in &self.fd_to_client {
            if let Err(err) = client.write(event) {
                log::error!("{err:?}");
                fds_to_drop.push(*fd);
            }
        }

        for fd in fds_to_drop {
            self.fd_to_client.remove(&fd);
        }
    }

    pub(crate) fn reply(&mut self, fd: i32, event: &Event) {
        let Some(client) = self.fd_to_client.get(&fd) else {
            return;
        };
        log::info!("Sending {event:?} to client {fd}..");

        if let Err(err) = client.write(event) {
            log::error!("{err:?}");
            self.fd_to_client.remove(&fd);
        }
    }

    pub(crate) fn remove_client(&mut self, fd: i32) {
        self.fd_to_client.remove(&fd);
    }

    pub(crate) fn contains_client(&self, fd: i32) -> bool {
        self.fd_to_client.contains_key(&fd)
    }
}

impl AsFd for Server {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for Server {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}
