#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{Config, StackResources};
use esp_hal::timer::timg::TimerGroup;
use panic_halt as _;
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let p = esp_hal::init(esp_hal::Config::default());
    let t = TimerGroup::new(p.TIMG0);
    esp_rtos::start(t.timer0, p.FROM_CPU_INTR0);

    let (wifi_ctrl, iface) = esp_radio::wifi::new(p.WIFI, Default::default())
        .expect("Failed to initialize WIFI controller");
    let net_conf = Config::dhcpv4(Default::default());

    let rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);

    let (stack, runner) = embassy_net::new(
        iface.station,
        net_conf,
        STACK_RESOURCES.init(StackResources::new()),
        seed);

    spawner.spawn(task_net(runner).unwrap());
}

#[embassy_executor::task]
async fn task_net(mut runner: Runner<'static, esp_radio::wifi::Interface<'static>>) -> ! {
    runner.run().await
}

