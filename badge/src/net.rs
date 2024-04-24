//! This example uses the RP Pico W board Wifi chip (cyw43).
//! Connects to specified Wifi network and creates a TCP endpoint on port 1234.

#![allow(async_fn_in_trait)]

extern crate alloc;
use core::future::Future;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use rand::SeedableRng;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const WIFI_NETWORK: &str = include_str!("../wifi.network.txt");
const WIFI_PASSWORD: &str = include_str!("../wifi.password.txt");

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[allow(non_snake_case)]
pub struct NetPins {
    pub PIN_23: PIN_23,
    pub PIN_25: PIN_25,
    pub PIO0: PIO0,
    pub PIN_24: PIN_24,
    pub PIN_29: PIN_29,
    pub DMA_CH0: DMA_CH0,
}

const FW: &[u8] = include_bytes!("../firmware/43439A0.bin");
const CLM: &[u8] = include_bytes!("../firmware/43439A0_clm.bin");

const CA: &str = include_str!("../../certs/CA_cert.crt");
const CERT: &str = include_str!("../../certs/client.crt");
const KEY: &str = include_str!("../../certs/client.key");

pub async fn main_net(
    p: NetPins,
    spawner: Spawner,
    badge_text: &mut impl FnMut(&str, bool),
    channel: &Signal<CriticalSectionRawMutex, u64>,
) -> Result<(), &'static str> {
    info!("Hello World!");

    badge_text("Starting net initialization", true);

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    //    status("Flashing cyw43 firmware");
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, FW).await;
    //   status("Spawning wifi task");
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(CLM).await;
    control
        .set_power_management(cyw43::PowerManagementMode::None)
        .await;

    let config = Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
    //});

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    //    status("Joining wpa2");
    loop {
        //control.join_open(WIFI_NETWORK).await;
        match control.join_wpa2(WIFI_NETWORK, WIFI_PASSWORD).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    badge_text("Waiting for DHCP", true);
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");
    badge_text("DHCP is now up!", true);

    let ipaddr = stack
        .config_v4()
        .ok_or_else(|| "no ipv4 address")?
        .address
        .address();

    //#[cfg(foobar)]

    let ca = pem::parse(CA).map_err(|_| "Failed to parse CA cert")?;
    let cert = pem::parse(CERT).map_err(|_| "Failed to parse client cert")?;
    let key = pem::parse(KEY).map_err(|_| "Failed to parse client key")?;

    use embedded_tls::{Certificate, TlsConfig, TlsConnection, TlsContext};

    let config = TlsConfig::new()
        .enable_rsa_signatures()
        .with_ca(Certificate::X509(ca.contents()))
        .with_priv_key(key.contents())
        .with_cert(Certificate::X509(cert.contents()));

    // And now we can use it!

    let mut rx_buffer = alloc::vec::Vec::new();
    let mut tx_buffer = alloc::vec::Vec::new();
    rx_buffer.resize(4096, 0u8);
    tx_buffer.resize(4096, 0u8);

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(Duration::from_secs(20)));

        const SERVER: &str = "dev.aughey.com";
        // Get address for dev.aughey.com through configured DNS
        let remote_host = match stack.dns_query(SERVER, DnsQueryType::A).await {
            Ok(addrs) => {
                if let Some(addr) = addrs.first() {
                    *addr
                } else {
                    badge_text("DNS query failed", true);
                    Timer::after(Duration::from_secs(3)).await;
                    continue;
                }
            }
            Err(e) => {
                badge_text("DNS query failed", true);
                Timer::after(Duration::from_secs(3)).await;
                continue;
            }
        };

        // Get address from 192.168.86.155
        // let remote_host = embassy_net::Ipv4Address::new(192, 168, 86, 155);

        match socket.connect((remote_host, 4444)).await {
            Ok(_) => {}
            Err(e) => {
                // sleep 3 seconds
                badge_text("Could not connect", true);
                Timer::after(Duration::from_secs(3)).await;
                continue;
            }
        }

        let mut read_buffer = alloc::vec::Vec::new();
        let mut write_buffer = alloc::vec::Vec::new();
        read_buffer.resize(16384, 0u8);
        write_buffer.resize(16384, 0u8);

        //static mut READ_RECORD_BUFFER: [u8; 16384] = [0u8; 16384];
        //static mut WRITE_RECORD_BUFFER: [u8; 16384] = [0u8; 16384];
        let mut tls = TlsConnection::new(
            socket,
            //embedded_io_adapters::std::FromStd::new(client),
            // unsafe { &mut READ_RECORD_BUFFER },
            // unsafe { &mut WRITE_RECORD_BUFFER },
            &mut read_buffer,
            &mut write_buffer,
        );

        if let Err(_) = tls
            .open(TlsContext::new(
                &config,
                embedded_tls::UnsecureProvider::new::<embedded_tls::Aes128GcmSha256>(
                    rand_chacha::ChaChaRng::seed_from_u64(1234),
                ),
            ))
            .await
        {
            badge_text("Failed to setup TLS connection", true);
            badge_text("Could not connect", true);
            Timer::after(Duration::from_secs(3)).await;
            continue;
        }

        //.map_err(|e| anyhow::anyhow!("Failed to open connection: {:?}", e))?;

        badge_text("TLS connection established!", true);

        let tls = EmbeddedAsyncWrapper(tls);

        if let Err(e) = handle_connection(tls, badge_text, channel).await {
            badge_text(e, true);
        }
    }
}

/// Will wait for a future to complete for duration time before returning an error.
async fn wait_timeout<F, V>(fut: F, duration: Duration) -> Result<V, &'static str>
where
    F: Future<Output = Result<V, &'static str>>,
{
    let timeout = Timer::after(duration);
    match wait_for_one_to_complete(core::pin::pin!(fut), timeout).await {
        FirstOrSecond::First(res) => res,
        FirstOrSecond::Second(_) => Err("Timeout"),
    }
}

async fn flush(io: &mut impl badge_net::AsyncWrite) -> Result<(), &'static str> {
    io.flush().await.map_err(|_| "Failed to flush")
}

async fn handle_connection<T>(
    mut tls: T,
    badge_text: &mut impl FnMut(&str, bool),
    channel: &Signal<CriticalSectionRawMutex, u64>,
) -> Result<(), &'static str>
where
    T: badge_net::AsyncRead + badge_net::AsyncWrite + Unpin,
{
    let mut buf = [0u8; 256];

    loop {
        // Send a request message
        wait_timeout(
            badge_net::write_frame(&mut tls, &badge_net::Request::Ready, &mut buf),
            Duration::from_secs(10),
        )
        .await?;

        wait_timeout(flush(&mut tls), Duration::from_secs(10))
            .await
            .map_err(|_| "Failed to flush")?;

        // Get a Update message
        let update = wait_timeout(
            badge_net::read_framed_value::<badge_net::Update>(&mut tls, &mut buf),
            Duration::from_secs(10),
        )
        .await?;

        if let Some(freq) = update.freq {
            channel.signal(freq);
        }

        if let Some(text) = update.text {
            badge_text(text, false);
        }
    }

    Ok(())
}

/// Return type of wait_for_one_to_complete indicating which future completed before the other.
pub enum FirstOrSecond<A, B> {
    First(A),
    Second(B),
}

/// Wait for one of the two futures to complete and return which one completed first.
/// This is a wrapper around the select function from the futures crate for the common
/// case of returning just an output item - dropping both futures at the completion of one.
pub async fn wait_for_one_to_complete<Fut1, Fut2, Out1, Out2>(
    fut1: Fut1,
    fut2: Fut2,
) -> FirstOrSecond<Out1, Out2>
where
    Fut1: Future<Output = Out1> + Unpin,
    Fut2: Future<Output = Out2> + Unpin,
{
    match embassy_futures::select::select(fut1, fut2).await {
        embassy_futures::select::Either::First(value_1) => FirstOrSecond::First(value_1),
        embassy_futures::select::Either::Second(value_2) => FirstOrSecond::Second(value_2),
    }
}

struct EmbeddedAsyncWrapper<T>(T);

impl<T> badge_net::AsyncRead for EmbeddedAsyncWrapper<T>
where
    T: embedded_io_async::Read + Unpin,
{
    type Error = embedded_io_async::ReadExactError<T::Error>;
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_exact(buf).await
    }
}
impl<T> badge_net::AsyncWrite for EmbeddedAsyncWrapper<T>
where
    T: embedded_io_async::Write + Unpin,
{
    type Error = T::Error;
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().await
    }
}
