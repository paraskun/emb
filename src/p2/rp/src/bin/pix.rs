#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::dma;
use embassy_rp::i2c;
use embassy_rp::i2c_slave;
use embassy_rp::peripherals::DMA_CH0;
use embassy_rp::peripherals::I2C0;
use embassy_rp::pio;
use embassy_rp::peripherals;
use panic_halt as _;

use p2::pal::pix::{Pix, Color};

const DEV_ADDR: u8 = 0x42;

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<peripherals::PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry",
)]
async fn main(_spawner: Spawner) {
    let hal = embassy_rp::init(Default::default());
    let mut pix = Pix::new(
        pio::Pio::new(hal.PIO0, Irqs),
        dma::Channel::new(hal.DMA_CH0, Irqs),
        hal.PIN_19);

    let mut conf = i2c_slave::Config::default();
    conf.addr = DEV_ADDR as u16;

    let mut dev = i2c_slave::I2cSlave::new(
        hal.I2C0,
        hal.PIN_5,
        hal.PIN_4,
        Irqs,
        conf);

    loop {
        let mut buf = [0u8; 4];

        match dev.listen(&mut buf).await {
            Err(i2c_slave::Error::PartialWrite(_)) => {
                pix.set(buf[0], Color::new(buf[1], buf[2], buf[3]));
                pix.push().await;
            }
            _ => (),
        }
    }
}
