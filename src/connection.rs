use pin_project_lite::pin_project;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream, ReadBuf};
use tokio::net::{TcpStream, ToSocketAddrs};

pin_project! {
    /// Connection wrapper
    #[derive(Debug)]
    #[must_use = "Connection do nothing unless polled"]
    pub struct Connection {
        #[pin]
        stream: BufStream<TcpStream>
    }
}

impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        self.project().stream.poll_read(cx, buf)
    }
}

impl AsyncWrite for Connection {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        self.project().stream.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.project().stream.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.project().stream.poll_shutdown(cx)
    }
}

impl AsyncBufRead for Connection {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<&[u8]>> {
        self.project().stream.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().stream.consume(amt)
    }
}

impl Connection {
    /// Connect to to given socket address
    pub async fn connect<A: ToSocketAddrs>(address: A) -> Result<Connection, io::Error> {
        TcpStream::connect(address).await.map(|c| Connection {
            stream: BufStream::new(c),
        })
    }

    /// Check if connection is broken by trying to read from it
    ///
    /// try_read()
    /// If data is successfully read, `Ok(n)` is returned, where `n` is the
    /// number of bytes read. `Ok(0)` indicates the stream's read half is closed
    /// and will no longer yield data. If the stream is not ready to read data
    /// `Err(io::ErrorKind::WouldBlock)` is returned.
    pub fn has_broken(&self) -> bool {
        self.stream
            .get_ref()
            .try_read(&mut []) // dirty way to try to read without buffer
            .map(|value| value == 0) // 0 indicates the stream's read half is closed
            .unwrap_or(true) // unwrap any error as true
    }

    /// Get reference to Stream
    pub fn get_ref(&self) -> &TcpStream {
        &self.stream.get_ref()
    }
}
