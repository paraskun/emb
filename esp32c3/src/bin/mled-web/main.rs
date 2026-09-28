#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{Config, Runner, StackResources};
use embassy_time::Timer;

use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::{AuthenticationMethodConfig};

use esp_backtrace as _;
use esp_println as _;
use esp_alloc as _;

// use defmt::{info, error};
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);

    let p = esp_hal::init(esp_hal::Config::default());
    let t = TimerGroup::new(p.TIMG0);
    esp_rtos::start(t.timer0, p.FROM_CPU_INTR0);

    esp_println::println!("Started.");

    let wifi_iface = esp_radio::wifi::Interface::station();
    let wifi_ctrl = esp_radio::wifi::WifiController::new(
        p.WIFI,
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
    spawner.spawn(task_wifi(wifi_ctrl).unwrap());

    esp_println::println!("Waiting for link...");

    stack.wait_config_up().await;
    if let Some(config) = stack.config_v4() {
        esp_println::println!("Got IP: {}", config.address);
    }

    esp_println::println!("Network configuration complete. Waiting.");

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn task_net(mut runner: Runner<'static, esp_radio::wifi::Interface>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn task_wifi(mut ctrl: esp_radio::wifi::WifiController<'static>) {
    let ssid = env!("SSID");
    let pass = env!("PASSWORD");

    let sta_conf = esp_radio::wifi::sta::StationConfig::default()
        .with_ssid(ssid.try_into().unwrap())
        .with_authentication(AuthenticationMethodConfig::WpaWpa2Personal(
            pass.try_into().unwrap()));
    let radio_conf = esp_radio::wifi::Config::Station(sta_conf);

    ctrl.set_config(&radio_conf).unwrap();

    loop {
        match ctrl.connect_async().await {
            Ok(_) => esp_println::println!("Wi-Fi connected."),
            Err(e) => {
                esp_println::println!("Failed to connect to Wi-Fi: {:?}.", e);
                Timer::after_secs(5).await;
                continue;
            }
        }

        let _ = ctrl.wait_for_disconnect_async().await;

        esp_println::println!("Wi-Fi disconnected. Reconnecting...");
    }
}

