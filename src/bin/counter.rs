#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::pio;
use embassy_rp::peripherals;
use embassy_time::Timer;
use embassy_futures::select;
use panic_halt as _;

use emb::pal::led::{Led, Digit};

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<peripherals::PIO0>;
});

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry",
)]
async fn main(_spawner: Spawner) {
    let hal = embassy_rp::init(Default::default());

    let mut sel_btn = gpio::Input::new(hal.PIN_19, gpio::Pull::Down);
    let mut inc_btn = gpio::Input::new(hal.PIN_20, gpio::Pull::Down);
    let mut cur = 0;

    let mut led = Led::new(pio::Pio::new(hal.PIO0, Irqs),
        hal.PIN_6,
        hal.PIN_11,
        hal.PIN_12,
        hal.PIN_7,
        hal.PIN_8,
        hal.PIN_9,
        hal.PIN_10,
    );

    led.ind[0] = true;
    led.set().await;

    loop {
        match select::select(
            sel_btn.wait_for_rising_edge(),
            inc_btn.wait_for_rising_edge(),
        ).await {
            select::Either::First(_) => {
                led.ind[cur] = false;
                cur = (cur + 1) % 4;
                led.ind[cur] = true;
            }
            select::Either::Second(_) => {
                led.dig[cur] += Digit::new(1);
            }
        }

        led.set().await;
        Timer::after_millis(200).await;
    }
}
