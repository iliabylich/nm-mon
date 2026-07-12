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

use anyhow::Result;
use pollsendd::Runtime;

mod args;
use args::{Args, RunMode};

mod dbus;
mod dbus_queue;
mod infallible_property;
mod nm;
mod service;

use service::NmMonService;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Always)
        .init();
    let args = Args::parse();
    log::trace!("Running with args {args:?}");

    let service = NmMonService::new()?;

    let mut rt = match args.mode {
        RunMode::Systemd => Runtime::new_with_systemd_socket(service),
        RunMode::Dev => Runtime::new_with_socket_path("/run/nm-mon-dev.sock", service),
    }?;

    loop {
        rt.poll()?;
    }
}
