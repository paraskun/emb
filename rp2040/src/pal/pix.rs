use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::dma;
use embassy_rp::pio;
use embassy_rp::Peri;
use embassy_rp::pio::ShiftConfig;
use fixed::types::U24F8;

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color(r, g, b)
    }

    pub fn red() -> Color {
        Color(10, 0, 0)
    }

    pub fn green() -> Color {
        Color(0, 10, 0)
    }

    pub fn blue() -> Color {
        Color(0, 0, 10)
    }
}

pub type Frame = [u32; 64];

pub struct Pix<'a, PIO: pio::Instance> {
    pio: pio::Pio<'a, PIO>,
    dma: dma::Channel<'a>,
    data: Frame,
}

impl<'a, PIO: pio::Instance> Pix<'a, PIO> {
    pub fn new(
        mut pio: pio::Pio<'a, PIO>,
        dma: dma::Channel<'a>,
        din: Peri<'a, impl pio::PioPin>,
    ) -> Pix<'a, PIO> {
        let din_pin = pio.common.make_pio_pin(din);

        let code = pio::program::pio_asm!("
        .side_set 1
        .wrap_target
        bit:
            out x, 1        side 0b0 [3]; 4
            jmp !x, zer     side 0b1 [2]; 3
        one:
            jmp bit         side 0b1 [2]; 3
        zer:
            nop             side 0b0 [2]; 3
        .wrap
        ");

        let mut cfg = pio::Config::default();
        cfg.use_program(&pio.common.load_program(&code.program), &[&din_pin]);
        cfg.clock_divider = U24F8::from_num(clk_sys_freq() / 1000) / U24F8::from_num(8000);
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            direction: pio::ShiftDirection::Left,
            threshold: 24,
        };

        pio.sm0.set_config(&cfg);
        pio.sm0.set_pin_dirs(pio::Direction::Out, &[&din_pin]);
        pio.sm0.set_enable(true);

        Pix {
            pio: pio,
            dma: dma,
            data: [0; 64],
        }
    }

    pub fn set(&mut self, i: u8, v: Color) {
        self.data[usize::from(i)] = (u32::from(v.1) << 24) | (u32::from(v.0) << 16) | (u32::from(v.2) << 8);
    }

    pub async fn push(&mut self) {
        self.pio.sm0.tx().dma_push(&mut self.dma, &self.data, false).await;
    }
}
