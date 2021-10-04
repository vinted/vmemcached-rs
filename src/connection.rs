use bytes::BytesMut;
use pin_project::pin_project;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufReader, BufWriter, ReadBuf};
use tokio::net::{TcpStream, ToSocketAddrs};

#[pin_project]
pub struct Connection {
    #[pin]
    stream: BufReader<BufWriter<TcpStream>>,
    // TODO: remove
    #[pin]
    buffer: BytesMut,
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
    pub async fn connect<A: ToSocketAddrs>(address: A) -> Result<Connection, io::Error> {
        TcpStream::connect(address).await.map(|c| Connection {
            stream: BufReader::new(BufWriter::new(c)),
            buffer: BytesMut::with_capacity(1024),
        })
    }

    pub fn buffer(self: Pin<&mut Self>) -> Pin<&mut BytesMut> {
        self.project().buffer
    }
}
