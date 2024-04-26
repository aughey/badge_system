use anyhow::Result;
// use rustls::crypto::{aws_lc_rs as provider, CryptoProvider};
// use rustls::server::WebPkiClientVerifier;
// use rustls::RootCertStore;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn server(
    _args: impl IntoIterator<Item = String>,
    get_frequency: impl Fn() -> Option<u32> + Send + 'static + Clone,
    get_text: impl Fn() -> Option<String> + Send + 'static + Clone,
) -> Result<()> {
    // let mut args = args.into_iter();
    // args.next();
    // let ca_file = args
    //     .next()
    //     .ok_or_else(|| anyhow::anyhow!("missing ca file argument"))?;
    // let cert_file = args
    //     .next()
    //     .ok_or_else(|| anyhow::anyhow!("missing certificate file argument"))?;
    // let private_key_file = args
    //     .next()
    //     .ok_or_else(|| anyhow::anyhow!("missing private key file argument"))?;

    // info!("Loading CA file: {:?}", ca_file);
    // let roots = {
    //     let mut filebuf = BufReader::new(
    //         File::open(&ca_file).map_err(|e| anyhow::anyhow!("Could not open {ca_file}: {e:?}"))?,
    //     );
    //     let root_ca = rustls_pemfile::certs(&mut filebuf);
    //     let mut roots = RootCertStore::empty();
    //     for cert in root_ca {
    //         roots
    //             .add(cert?)
    //             .map_err(|_| anyhow::anyhow!("failed to add cert to root store"))?;
    //     }
    //     roots
    // };

    // info!("Loading cert file: {:?}", cert_file);
    // let certs = rustls_pemfile::certs(&mut BufReader::new(
    //     &mut File::open(&cert_file)
    //         .map_err(|e| anyhow::anyhow!("Could not open {cert_file}: {e:?}"))?,
    // ))
    // .collect::<Result<Vec<_>, _>>()?;
    // info!("Loading private key file: {:?}", private_key_file);
    // let private_key =
    //     rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(&private_key_file)?))?
    //         .ok_or_else(|| {
    //             anyhow::anyhow!(
    //                 "no private key found in {:?} (encrypted keys not supported)",
    //                 private_key_file
    //             )
    //         })?;

    // info!("Building client verifier");
    // let client_verifier = WebPkiClientVerifier::builder(roots.into()).build()?;

    // let config = rustls::ServerConfig::builder_with_provider(
    //     CryptoProvider {
    //         cipher_suites: [
    //             provider::cipher_suite::TLS13_AES_256_GCM_SHA384,
    //             provider::cipher_suite::TLS13_AES_128_GCM_SHA256,
    //             provider::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
    //         ]
    //         .to_vec(),
    //         ..provider::default_provider()
    //     }
    //     .into(),
    // )
    // .with_protocol_versions(&[&rustls::version::TLS13])?
    // .with_client_cert_verifier(client_verifier)
    // .with_single_cert(certs, private_key)?;
    //let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(format!("[::]:{}", 4443)).await?;

    loop {
        info!("Waiting for incoming connection");
        let (stream, _) = listener.accept().await?;
        info!("Got incoming connection");

        // turn stream into an async stream
        // let mut stream = match acceptor.accept(stream).await {
        //     Ok(stream) => stream,
        //     Err(e) => {
        //         eprintln!("accept error: {:?}", e);
        //         continue;
        //     }
        // };

        let stream = ReadWriteWrapper { inner: stream };

        let get_frequency = get_frequency.clone();
        let get_text = get_text.clone();
        tokio::spawn(async move {
            match handle_connection(stream, get_frequency, get_text).await {
                Ok(_) => info!("Connection handled successfully"),
                Err(e) => error!("Error handling connection: {:?}", e),
            }
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

struct ReadWriteWrapper<T> {
    inner: T,
}
impl<T> badge_net::AsyncRead for ReadWriteWrapper<T>
where
    T: AsyncReadExt + Unpin,
{
    type Error = tokio::io::Error;
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.read_exact(buf).await?;
        Ok(())
    }
}
impl<T> badge_net::AsyncWrite for ReadWriteWrapper<T>
where
    T: AsyncWriteExt + Unpin,
{
    type Error = tokio::io::Error;
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.inner.write_all(buf).await?;
        Ok(())
    }
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush().await?;
        Ok(())
    }
}

async fn handle_connection<C>(
    mut stream: C,
    get_rate: impl Fn() -> Option<u32>,
    get_text: impl Fn() -> Option<String>,
) -> Result<()>
where
    C: badge_net::AsyncRead + badge_net::AsyncWrite + Unpin,
{
    info!("Reading from stream");
    let mut count = 0u32;

    let mut last_text = None;
    let mut last_freq = None;

    loop {
        let mut buf = [0u8; 256];

        count = count.wrapping_add(1);

        let request =
            badge_net::read_framed_value::<badge_net::Request>(&mut stream, buf.as_mut_slice())
                .await
                .map_err(anyhow::Error::msg)?;
        if request == badge_net::Request::Close {
            break;
        }

        // sleep 3 seconds
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let text = {
            let thistext = get_text();
            if last_text != thistext {
                last_text = thistext.clone();
                thistext
            } else {
                None
            }
        };

        let freq = {
            let thisfreq = get_rate();
            if last_freq != thisfreq {
                last_freq = thisfreq.clone();
                thisfreq
            } else {
                None
            }
        };

        //info!("Sending badge count {count}");
        badge_net::write_frame(
            &mut stream,
            &badge_net::Update {
                text: text.as_ref().map(|x| x.as_str()),
                freq: freq,
            },
            buf.as_mut_slice(),
        )
        .await
        .map_err(anyhow::Error::msg)?;
        stream
            .flush()
            .await
            .map_err(|_| anyhow::anyhow!("Failed to flush"))?;
    }

    Ok(())
}
