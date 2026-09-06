#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::pio;
use embassy_rp::peripherals;
use embassy_time::Timer;
use panic_halt as _;

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<peripherals::PIO0>;
});

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry",
)]
async fn main(_spawner: Spawner) {
    let hal = embassy_rp::init(Default::default());
    let pio::Pio {
        mut common,
        mut sm0,
        ..
    } = pio::Pio::new(hal.PIO0, Irqs);

    let prg = pio::program::pio_asm!("
    set pindirs, 1
    .wrap_target
        set pins, 1 [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        set pins, 0 [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
    .wrap
    ");

    let out_pin = common.make_pio_pin(hal.PIN_25);

    let mut cfg = pio::Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.set_set_pins(&[&out_pin]);
    cfg.clock_divider = 65535u16.into();

    sm0.set_config(&cfg);
    sm0.set_enable(true);

    loop {
        Timer::after_secs(1).await;
    }
}
