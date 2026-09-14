use embassy_rp::pio;
use embassy_rp::Peri;

use core::convert::From;
use core::convert::Into;
use core::default::Default;
use core::ops::{Add, AddAssign, Rem};
use core::option::Option::None;
use core::iter::Iterator;

#[derive(Clone, Copy)]
pub struct Digit(u8);

impl Digit {
    pub fn new(v: u8) -> Digit {
        Digit(v % 10)
    }
}

impl Add for Digit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Digit(self.0 + rhs.0)
    }
}

impl AddAssign for Digit {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (self.0 + rhs.0) % 10;
    }
}

impl Rem for Digit {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Digit(self.0 % rhs.0)
    }
}

impl From<Digit> for usize {
    fn from(value: Digit) -> Self {
        return value.0 as usize;
    }
}

const MAP: [u8; 10] = [
//  0babcdefgP
    0b11111100, // 0
    0b01100000, // 1
    0b11011010, // 2
    0b11110010, // 3
    0b01100110, // 4
    0b10110110, // 5
    0b10111110, // 6
    0b11100000, // 7
    0b11111110, // 8
    0b11110110, // 9
];

pub struct Led<'a, PIO: pio::Instance> {
    pio: pio::Pio<'a, PIO>,

    pub dig: [Digit; 4],
    pub ind: [bool; 4],
}

impl<'a, PIO: pio::Instance> Led<'a, PIO> {
    pub fn new(
        mut pio: pio::Pio<'a, PIO>,
    
        dat: Peri<'a, impl pio::PioPin>,
        clk: Peri<'a, impl pio::PioPin>,
        lat: Peri<'a, impl pio::PioPin>,
    
        d1: Peri<'a, impl pio::PioPin>,
        d2: Peri<'a, impl pio::PioPin>,
        d3: Peri<'a, impl pio::PioPin>,
        d4: Peri<'a, impl pio::PioPin>,
    ) -> Led<'a, PIO> {
        let dat_pin = pio.common.make_pio_pin(dat);
        let clk_pin = pio.common.make_pio_pin(clk);
        let lat_pin = pio.common.make_pio_pin(lat);
    
        let d1_pin = pio.common.make_pio_pin(d1);
        let d2_pin = pio.common.make_pio_pin(d2);
        let d3_pin = pio.common.make_pio_pin(d3);
        let d4_pin = pio.common.make_pio_pin(d4);
    
        let prg = pio::program::pio_asm!("
        .side_set 2
        .wrap_target
            pull noblock        side 0b00 [1]
            mov x, osr          side 0b00 [1]
        
            set pins, 0b1111    side 0b00 [1]
            set y, 7            side 0b00 [1]
        loop1:
            out pins, 1         side 0b00 [1]
            nop                 side 0b01 [1] ; clock
            jmp y-- loop1       side 0b00 [1]
            nop                 side 0b10 [1] ; latch
            set pins, 0b0111    side 0b00 [1]
        
            set pins, 0b1111    side 0b00 [1]
            set y, 7            side 0b00 [1]
        loop2:
            out pins, 1         side 0b00 [1]
            nop                 side 0b01 [1] ; clock
            jmp y-- loop2       side 0b00 [1]
            nop                 side 0b10 [1] ; latch
            set pins, 0b1011    side 0b00 [1]
        
            set pins, 0b1111    side 0b00 [1]
            set y, 7            side 0b00 [1]
        loop3:
            out pins, 1         side 0b00 [1]
            nop                 side 0b01 [1] ; clock
            jmp y-- loop3       side 0b00 [1]
            nop                 side 0b10 [1] ; latch
            set pins, 0b1101    side 0b00 [1]
        
            set pins, 0b1111    side 0b00 [1]
            set y, 7            side 0b00 [1]
        loop4:
            out pins, 1         side 0b00 [1]
            nop                 side 0b01 [1] ; clock
            jmp y-- loop4       side 0b00 [1]
            nop                 side 0b10 [1] ; latch
            set pins, 0b1110    side 0b00 [1]
        .wrap
        ");
    
        let mut cfg = pio::Config::default();
        cfg.use_program(&pio.common.load_program(&prg.program), &[&clk_pin, &lat_pin]);
        cfg.set_out_pins(&[&dat_pin]);
        cfg.set_set_pins(&[&d1_pin, &d2_pin, &d3_pin, &d4_pin]);
        cfg.clock_divider = 200u16.into();
    
        pio.sm0.set_config(&cfg);
        pio.sm0.set_pin_dirs(pio::Direction::Out, &[
            &dat_pin,
            &clk_pin,
            &lat_pin,
            &d1_pin,
            &d2_pin,
            &d3_pin,
            &d4_pin,
        ]);
        pio.sm0.set_enable(true);
    
        Led {
            pio: pio,
            dig: [Digit(0), Digit(0), Digit(0), Digit(0)],
            ind: [false, false, false, false],
        }
    }

    pub async fn set(&mut self) {
        let mut pinout: u32 = 0;

        for (i, &d) in self.dig.iter().enumerate() {
            pinout |= (MAP[usize::from(d)] as u32) << (8 * (3 - i));
        }

        for (i, &p) in self.ind.iter().enumerate() {
            pinout |= (if p { 1u32 } else { 0u32 }) << (8 * (3 - i));
        }

        self.pio.sm0.tx().wait_push(pinout).await;
    }
}
