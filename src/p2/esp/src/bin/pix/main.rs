#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{Config, Runner, StackResources, tcp::TcpSocket};
use embassy_time::{Duration, Timer};

use esp_hal::{time::Rate, timer::timg::TimerGroup};
use esp_radio::wifi::{AuthenticationMethodConfig};
use esp_hal::i2c;
use esp_backtrace as _;
use esp_println as _;
use esp_alloc as _;

use static_cell::StaticCell;
use embedded_io_async::Write;

static DEV_ADDR: u8 = 0x42;
static DIV: u8 = 15;
static PAGE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html.gz"));
static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);

    let hal = esp_hal::init(esp_hal::Config::default());
    let tg = TimerGroup::new(hal.TIMG0);
    esp_rtos::start(tg.timer0, hal.FROM_CPU_INTR0);

    let wifi_iface = esp_radio::wifi::Interface::station();
    let wifi_ctl = esp_radio::wifi::WifiController::new(
        hal.WIFI,
        Default::default()).unwrap();
    let net_conf = Config::dhcpv4(Default::default());

    let rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);

    let (stack, runner) = embassy_net::new(
        wifi_iface,
        net_conf,
        STACK_RESOURCES.init(StackResources::new()),
        seed);

    spawner.spawn(task_net(runner).unwrap());
    spawner.spawn(task_wifi(wifi_ctl).unwrap());

    stack.wait_config_up().await;
    if let Some(config) = stack.config_v4() {
        esp_println::println!("Got IP: {}", config.address);
    }

    let mut i2c_ctl = i2c::master::I2c::new(
        hal.I2C0,
        i2c::master::Config::default()
            .with_frequency(Rate::from_khz(100)))
        .unwrap()
        .with_scl(hal.GPIO9)
        .with_sda(hal.GPIO8)
        .into_async();

    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];

    loop {
        let mut sock = TcpSocket::new(stack, &mut rx, &mut tx);
        sock.set_timeout(Some(Duration::from_secs(3)));

        if let Err(_) = sock.accept(80).await {
            continue;
        }

        let mut buf = [0u8; 1024];
        let mut len = 0;

        loop {
            match sock.read(&mut buf[len..]).await {
                Ok(0) => break,
                Ok(n) => {
                    len += n;

                    if buf[..len].windows(4).any(|w| w == b"\r\n\r\n") || len == buf.len() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        let req = core::str::from_utf8(&buf[..len]).unwrap_or("");
        let path = req.split_whitespace().nth(1).unwrap_or("/");

        match path {
            "/" => {
                let mut hdr = [0u8; 128];
                let n = fmt_200(&mut hdr, PAGE.len());
                sock.write_all(&hdr[..n]);
                sock.write_all(PAGE);
            }
            _ => {
                if path.starts_with("/set") {
                    let mut tok = path.split("/");

                    let _ = tok.next();
                    let _ = tok.next();
                    let i = tok.next().unwrap().parse::<u8>().ok().unwrap();
                    let r = tok.next().unwrap().parse::<u8>().ok().unwrap();
                    let g = tok.next().unwrap().parse::<u8>().ok().unwrap();
                    let b = tok.next().unwrap().parse::<u8>().ok().unwrap();

                    sock.write_all("
                        HTTP/1.1 204 No Content\r\n\
                        Connection: close\r\n\
                        \r\n".as_bytes()).await;

                    let _ = i2c_ctl.write_async(DEV_ADDR, &[i, r / DIV, g / DIV, b / DIV]).await;
                }
            }
        }

        let _ = sock.flush().await;
        sock.close();
    }
}

fn fmt_200(buf: &mut [u8], content_len: usize) -> usize {
    use core::fmt::Write;
    let mut w = BufWriter { buf, pos: 0 };
    write!(w, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Encoding: gzip\r\nContent-Length: {content_len}\r\nConnection: close\r\n\r\n").unwrap();
    w.pos
}

struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl core::fmt::Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}

#[embassy_executor::task]
async fn task_net(mut runner: Runner<'static, esp_radio::wifi::Interface>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn task_wifi(mut ctl: esp_radio::wifi::WifiController<'static>) {
    let ssid = env!("SSID");
    let pass = env!("PASSWORD");

    let sta_conf = esp_radio::wifi::sta::StationConfig::default()
        .with_ssid(ssid.try_into().unwrap())
        .with_authentication(AuthenticationMethodConfig::WpaWpa2Personal(
            pass.try_into().unwrap()));
    let radio_conf = esp_radio::wifi::Config::Station(sta_conf);

    ctl.set_config(&radio_conf).unwrap();

    loop {
        match ctl.connect_async().await {
            Ok(_) => {},
            Err(_) => {
                Timer::after_secs(5).await;
                continue;
            }
        }

        let _ = ctl.wait_for_disconnect_async().await;
    }
}
