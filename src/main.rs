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

use std::os::fd::AsRawFd;

use anyhow::{Result, bail};
use rustix::event::PollFlags;

mod args;
use args::Args;

mod dbus_queue;
use dbus_queue::DBusQueue;

mod infallible_property;

mod nm;
use nm::NM;

mod dbus;
use dbus::DBus;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Always)
        .init();
    let args = Args::parse();
    log::trace!("Running with args {args:?}");

    let mut queue = DBusQueue::new()?;
    let mut nm = NM::new(&mut queue);

    let mut dbus = DBus::new()?;
    let dbusfd = dbus.as_raw_fd();
    let mut readbuf = vec![0; 10 * 1_024];

    loop {
        let mut pollfds = [dbus.as_pollfd(&mut readbuf, &queue)?];
        rustix::event::poll(&mut pollfds, None)?;
        let Some(revents) = REvents::new(dbusfd, pollfds[0].revents())? else {
            bail!("poll() didn't fill any pollfd");
        };

        if revents.readable
            && let Some(message) = dbus.read(&mut readbuf, &queue)?
        {
            let mut events = vec![];
            nm.handle(message, &mut events, &mut queue);
            log::info!("{events:?}");
        }
        if revents.writable {
            dbus.write(&mut readbuf, &mut queue)?;
        }
    }
}

struct REvents {
    readable: bool,
    writable: bool,
}
impl REvents {
    fn new(fd: i32, revents: PollFlags) -> Result<Option<Self>> {
        if revents.intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL) {
            bail!("FD {fd} returned revents {revents:?}");
        }
        let readable = revents.contains(PollFlags::IN);
        let writable = revents.contains(PollFlags::OUT);
        if readable || writable {
            Ok(Some(Self { readable, writable }))
        } else {
            Ok(None)
        }
    }
}
