#![no_std]
#![no_main]

use core::{fmt::{self, Write}, sync::atomic::{AtomicU32, Ordering::SeqCst}};

use embassy_executor::Spawner;
use embassy_rp::{clocks::clk_sys_freq, config::Config, gpio::{Input, Pull}, i2c::{I2c}, peripherals, pio::{self, Direction::Out, ShiftConfig, ShiftDirection::Left}};
use embassy_rp::i2c;
use embassy_time::Timer;
use embedded_graphics::{Drawable, geometry::Point, mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10}, pixelcolor::BinaryColor, text::{Baseline, Text}};
use fixed::types::U24F8;
use panic_halt as _;
use ssd1306::{I2CDisplayInterface, Ssd1306Async, rotation::DisplayRotation, size::DisplaySize128x64, mode::{DisplayConfigAsync}};

static DOUT: AtomicU32 = AtomicU32::new(0);

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<peripherals::PIO0>;
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
});

struct ArrayBuffer<'a> {
    buf: &'a mut [u8],
    cursor: usize,
}

impl<'a> ArrayBuffer<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, cursor: 0 }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.cursor]).unwrap_or("")
    }
}

// Реализуем Write, чтобы работал макрос write!
impl<'a> Write for ArrayBuffer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remain = self.buf.len() - self.cursor;
        
        if bytes.len() > remain {
            return Err(fmt::Error); // Буфер переполнен
        }
        
        self.buf[self.cursor..self.cursor + bytes.len()].copy_from_slice(bytes);
        self.cursor += bytes.len();
        Ok(())
    }
}

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry",
)]
async fn main(spawner: Spawner) {
    let hal = embassy_rp::init(Config::default());

    let mut pio0 = pio::Pio::new(hal.PIO0, Irqs);

    let code = pio::program::pio_file!("src/ws2812.pio");
    let din = pio0.common.make_pio_pin(hal.PIN_19);

    let mut cfg = pio::Config::default();
    cfg.use_program(&pio0.common.load_program(&code.program), &[&din]);
    cfg.clock_divider = U24F8::from_num(clk_sys_freq() / 1000) / U24F8::from_num(8000);
    cfg.fifo_join = pio::FifoJoin::TxOnly;
    cfg.shift_out = ShiftConfig {
        auto_fill: true,
        threshold: 24,
        direction: Left,
    };

    pio0.sm0.set_config(&cfg);
    pio0.sm0.set_pin_dirs(Out, &[&din]);
    pio0.sm0.set_enable(true);

    spawner.spawn(task_irq(Input::new(hal.PIN_0, Pull::Down))
        .unwrap());

    let v: u32 = (5 << 24) | (5 << 16) | (5 << 8);

    for _ in 0..128 {
        pio0.sm0.tx().wait_push(v).await;
    }

    Timer::after_millis(100).await;

    let sda = hal.PIN_16;
    let scl = hal.PIN_17;

    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = 400_000;

    let i2c_bus = I2c::new_async(hal.I2C0, scl, sda, Irqs, i2c_config);
    let i2c_interface = I2CDisplayInterface::new(i2c_bus);
    let mut display = Ssd1306Async::new(i2c_interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init()
        .await
        .expect("failed to initialize the display");

    let text_style = MonoTextStyleBuilder::new()
       .font(&FONT_6X10)
       .text_color(BinaryColor::On)
       .build();

    let mut raw = [0u8; 64];
    let mut buf = ArrayBuffer::new(&mut raw);

    let _ = write!(buf, "Size: {}", DOUT.load(SeqCst));

    Text::with_baseline(buf.as_str(), Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .expect("failed to draw text to display");

    display
        .flush()
        .await
        .expect("failed to flush data to display");

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn task_irq(mut input: Input<'static>) {
    loop {
        input.wait_for_rising_edge().await;
        DOUT.store(DOUT.load(SeqCst) + 1, SeqCst);
    }
}