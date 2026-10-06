#![no_std]
#![no_main]

mod pio;
mod usb;

use embassy_executor::Spawner;
use embassy_rp::{
    config::Config,
    i2c, peripherals,
    pio as rp_pio,
    usb::Driver,
};
use embassy_usb::UsbDevice;
use panic_halt as _;

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => rp_pio::InterruptHandler<peripherals::PIO0>;
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<peripherals::USB>;
});

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, Driver<'static, peripherals::USB>>) {
    device.run().await;
}

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry",
)]
async fn main(spawner: Spawner) {
    let hal = embassy_rp::init(Config::default());

    let pio0 = rp_pio::Pio::new(hal.PIO0, Irqs);
    let mut screen = pio::Screen::new(pio0, hal.PIN_19, hal.PWM_SLICE0, hal.PIN_1);

    let (usb_dev, mut serial) = usb::new(hal.USB, Irqs);
    spawner.spawn(usb_task(usb_dev).unwrap());

    loop {
        match serial.read().await {
            usb::Cmd::Size => {
                let size = screen.size().await;
                serial.write(format_args!("{}\n", size)).await
            },
            usb::Cmd::Unknown => serial.write(format_args!("unknown\n")).await,
        }
    }
}
