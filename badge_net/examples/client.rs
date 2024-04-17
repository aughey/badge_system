use anyhow::Result;
use embedded_tls::{Certificate, TlsConfig, TlsConnection, TlsContext};
use rand::rngs::OsRng;

use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let client = TcpStream::connect("127.0.0.1:4443").await?;

    let ca = include_str!("../../certs/CA_cert.crt");
    let ca = pem_parser::pem_to_der(ca);

    let cert = include_str!("../../certs/client.crt");
    let cert = pem_parser::pem_to_der(cert);

    let key = include_str!("../../certs/client.key");
    let key = pem_parser::pem_to_der(key);

    let config = TlsConfig::new()
        .enable_rsa_signatures()
        .with_ca(Certificate::X509(&ca))
        .with_priv_key(&key)
        .with_cert(Certificate::X509(&cert));
    let mut read_record_buffer = [0u8; 16384];
    let mut write_record_buffer = [0u8; 16384];
    let mut tls = TlsConnection::new(
        FromTokio::new(client),
        //embedded_io_adapters::std::FromStd::new(client),
        &mut read_record_buffer,
        &mut write_record_buffer,
    );

    tls.open(TlsContext::new(
        &config,
        embedded_tls::UnsecureProvider::new::<embedded_tls::Aes128GcmSha256>(OsRng),
    ))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to open connection: {:?}", e))?;

    println!("TLS connection established!");

    let mut buf = [0u8; 1024];
    let size = tls
        .read(&mut buf)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read data: {:?}", e))?;
    let buf = std::str::from_utf8(&buf[..size])?;
    println!("read {} bytes: {}", size, buf);

    tls.write("Hello World from client".as_bytes())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write data: {:?}", e))?;

    tls.close()
        .await
        .map_err(|(_, e)| anyhow::anyhow!("Failed to close connection: {:?}", e))?;

    Ok(())
}

use core::future::poll_fn;
use core::pin::Pin;
use core::task::Poll;
use tokio::io::AsyncBufReadExt;

/// Adapter from `tokio::io` traits.
#[derive(Clone)]
pub struct FromTokio<T: ?Sized> {
    inner: T,
}

#[allow(dead_code)]
impl<T> FromTokio<T> {
    /// Create a new adapter.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Consume the adapter, returning the inner object.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[allow(dead_code)]
impl<T: ?Sized> FromTokio<T> {
    /// Borrow the inner object.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Mutably borrow the inner object.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: ?Sized> embedded_io::ErrorType for FromTokio<T> {
    type Error = std::io::Error;
}

impl<T: tokio::io::AsyncRead + Unpin + ?Sized> embedded_io_async::Read for FromTokio<T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // The current tokio implementation (https://github.com/tokio-rs/tokio/blob/tokio-1.33.0/tokio/src/io/poll_evented.rs#L165)
        // does not consider the case of buf.is_empty() as a special case,
        // which can cause Poll::Pending to be returned at the end of the stream when called with an empty buffer.
        // This poll will, however, never become ready, as no more bytes will be received.
        if buf.is_empty() {
            return Ok(0);
        }

        poll_fn(|cx| {
            let mut buf = tokio::io::ReadBuf::new(buf);
            match Pin::new(&mut self.inner).poll_read(cx, &mut buf) {
                Poll::Ready(r) => match r {
                    Ok(()) => Poll::Ready(Ok(buf.filled().len())),
                    Err(e) => Poll::Ready(Err(e)),
                },
                Poll::Pending => Poll::Pending,
            }
        })
        .await
    }
}

impl<T: tokio::io::AsyncBufRead + Unpin + ?Sized> embedded_io_async::BufRead for FromTokio<T> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        self.inner.fill_buf().await
    }

    fn consume(&mut self, amt: usize) {
        Pin::new(&mut self.inner).consume(amt);
    }
}

impl<T: tokio::io::AsyncWrite + Unpin + ?Sized> embedded_io_async::Write for FromTokio<T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        match poll_fn(|cx| Pin::new(&mut self.inner).poll_write(cx, buf)).await {
            Ok(0) if !buf.is_empty() => Err(std::io::ErrorKind::WriteZero.into()),
            Ok(n) => Ok(n),
            Err(e) => Err(e),
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        poll_fn(|cx| Pin::new(&mut self.inner).poll_flush(cx)).await
    }
}

impl<T: tokio::io::AsyncSeek + Unpin + ?Sized> embedded_io_async::Seek for FromTokio<T> {
    async fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
        // Note: `start_seek` can return an error if there is another seek in progress.
        // Therefor it is recommended to call `poll_complete` before any call to `start_seek`.
        poll_fn(|cx| Pin::new(&mut self.inner).poll_complete(cx)).await?;
        Pin::new(&mut self.inner).start_seek(pos.into())?;
        poll_fn(|cx| Pin::new(&mut self.inner).poll_complete(cx)).await
    }
}

// TODO: ToTokio.
// It's a bit tricky because tokio::io is "stateless", while we're "stateful" (we
// return futures that borrow Self and get polled for the duration of the operation.)
// It can probably done by storing the futures in Self, with unsafe Pin hacks because
// we're a self-referential struct
