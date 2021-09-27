use std::io::{self, Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(feature = "tls")]
use openssl::ssl::SslStream;

/// Stream wrapper
#[allow(missing_debug_implementations)]
pub enum Stream {
    /// TCP stream
    Tcp(TcpStream),
    #[cfg(unix)]
    /// Unix socket stream
    Unix(UnixStream),
    /// TLS stream
    #[cfg(feature = "tls")]
    Tls(SslStream<TcpStream>),
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Stream::Tcp(ref mut stream) => stream.read(buf),
            #[cfg(unix)]
            Stream::Unix(ref mut stream) => stream.read(buf),
            #[cfg(feature = "tls")]
            Stream::Tls(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Tcp(ref mut stream) => stream.write(buf),
            #[cfg(unix)]
            Stream::Unix(ref mut stream) => stream.write(buf),
            #[cfg(feature = "tls")]
            Stream::Tls(ref mut stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Stream::Tcp(ref mut stream) => stream.flush(),
            #[cfg(unix)]
            Stream::Unix(ref mut stream) => stream.flush(),
            #[cfg(feature = "tls")]
            Stream::Tls(ref mut stream) => stream.flush(),
        }
    }
}
