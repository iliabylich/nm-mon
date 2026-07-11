#![forbid(unsafe_code)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(deprecated_in_future)]
#![warn(unused_lifetimes)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use anyhow::{Context, Result};
use nm_mon::Event;
use rustix::event::PollFlags;
use std::os::fd::{AsFd, AsRawFd};

mod args;
use args::Args;

mod dbus_queue;
use dbus_queue::DBusQueue;

mod infallible_property;

mod nm;
use nm::NM;

mod dbus;
use dbus::DBus;

mod server;
use server::Server;

mod client;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Always)
        .init();
    let args = Args::parse();
    log::trace!("Running with args {args:?}");

    let mut queue = DBusQueue::new()?;
    let mut nm = NM::new(&mut queue);

    let mut dbus = DBus::new()?;
    let mut readbuf = vec![0; 10 * 1_024];

    let mut server = Server::new(&args)?;

    let mut last_ssid_and_strength_event = None;

    loop {
        let mut pollfds = server
            .pollfds()
            .chain(std::iter::once(dbus.as_pollfd(&mut readbuf, &queue)?))
            .collect::<Vec<_>>();
        rustix::event::poll(&mut pollfds, None)?;

        let fd_to_readable_writable = pollfds
            .into_iter()
            .map(|pollfd| {
                let fd = pollfd.as_fd().as_raw_fd();
                let revents = pollfd.revents();

                if revents.intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL) {
                    return (fd, Err(anyhow::anyhow!("fd returned error: {revents:?}")));
                }

                (
                    fd,
                    Ok((
                        revents.contains(PollFlags::IN),
                        revents.contains(PollFlags::OUT),
                    )),
                )
            })
            .collect::<Vec<_>>();

        for (fd, readable_writable) in fd_to_readable_writable {
            if fd == server.as_raw_fd() {
                let (readable, _writable) = readable_writable.context("serverfd failed")?;
                if readable {
                    log::trace!("server is readable");
                    server.accept()?;
                }
            } else if fd == dbus.as_raw_fd() {
                let (readable, writable) = readable_writable.context("dbusfd failed")?;

                if readable && let Some(message) = dbus.read(&mut readbuf, &queue)? {
                    let mut events = vec![];
                    nm.handle(message, &mut events, &mut queue);
                    log::trace!("{events:?}");
                    for event in events {
                        server.broadcast(&event);
                        if matches!(event, Event::SsidAndStrength { .. }) {
                            last_ssid_and_strength_event = Some(event);
                        }
                    }
                }
                if writable {
                    dbus.write(&mut readbuf, &mut queue)?;
                }
            } else if server.contains_client(fd) {
                match readable_writable {
                    Ok((readable, _writable)) => {
                        if readable
                            && server.read_client(fd)
                            && let Some(event) = &last_ssid_and_strength_event
                        {
                            server.write_client(fd, event);
                        }
                    }
                    Err(err) => {
                        log::error!("clientfd {fd} failed, removing it...");
                        log::error!("{err:?}");
                        server.remove_client(fd);
                    }
                }
            }
        }
    }
}
