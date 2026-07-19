use anyhow::{Context as _, Result};
use nm_mon::Event;
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};

const USAGE: &str = "
Usage: cli /path/to/socket
";
fn print_usage_and_exit() -> ! {
    eprintln!("{USAGE}");
    std::process::exit(1);
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| {
        eprintln!("no path given");
        print_usage_and_exit()
    });

    let fd = rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None)
        .context("socket() failed")?;

    let addr = SocketAddrUnix::new(path).context("can't build UNIX socket address")?;
    rustix::net::connect(&fd, &addr).context("connect() failed")?;

    loop {
        let mut buf = [0; Event::SERIALIZED_LENGTH];
        let len = rustix::io::read(&fd, &mut buf)?;
        assert!(len == buf.len());
        let event = Event::deserialize(buf);
        println!("{event:?}");
    }
}
