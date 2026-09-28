#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;

use esp_backtrace as _;
use esp_println as _;
use esp_alloc as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    let p = esp_hal::init(esp_hal::Config::default());

    let t = TimerGroup::new(p.TIMG0);
    esp_rtos::start(t.timer0, p.FROM_CPU_INTR0);

    let mut led = Output::new(p.GPIO0, Level::Low, OutputConfig::default());

    loop {
        led.toggle();
        Timer::after_secs(1).await;
    }
}
