#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::pio;
use embassy_rp::peripherals;
use embassy_time::Timer;
use embassy_futures::select;
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
    let map: [u8; 10] = [
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

    let mut select_btn = gpio::Input::new(hal.PIN_19, gpio::Pull::Down);
    let mut count_btn = gpio::Input::new(hal.PIN_20, gpio::Pull::Down);

    let mut d: [u8; 4] = [0, 0, 0, 0];
    let mut current = 0;

    let pio::Pio {
        mut common,
        mut sm0,
        ..
    } = pio::Pio::new(hal.PIO0, Irqs);

    let dat_pin = common.make_pio_pin(hal.PIN_6);
    let clk_pin = common.make_pio_pin(hal.PIN_11);
    let lat_pin = common.make_pio_pin(hal.PIN_12);

    let d1_pin = common.make_pio_pin(hal.PIN_7);
    let d2_pin = common.make_pio_pin(hal.PIN_8);
    let d3_pin = common.make_pio_pin(hal.PIN_9);
    let d4_pin = common.make_pio_pin(hal.PIN_10);

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
    cfg.use_program(&common.load_program(&prg.program), &[&clk_pin, &lat_pin]);
    cfg.set_out_pins(&[&dat_pin]);
    cfg.set_set_pins(&[
        &d1_pin,
        &d2_pin,
        &d3_pin,
        &d4_pin,
    ]);

    cfg.clock_divider = 200u16.into();
    cfg.shift_out.auto_fill = true;

    sm0.set_config(&cfg);
    sm0.set_pin_dirs(pio::Direction::Out, &[
        &clk_pin,
        &lat_pin,
        &dat_pin,
        &d1_pin,
        &d2_pin,
        &d3_pin,
        &d4_pin,
    ]);
    sm0.set_enable(true);

    let mut pinout: u32 = 0;
    pinout |= (map[usize::from(d[0])] as u32) << 24;
    pinout |= (map[usize::from(d[1])] as u32) << 16;
    pinout |= (map[usize::from(d[2])] as u32) << 8;
    pinout |= (map[usize::from(d[3])] as u32) << 0;

    sm0.tx().wait_push(pinout).await;

    loop {
        match select::select(
            select_btn.wait_for_rising_edge(),
            count_btn.wait_for_rising_edge(),
        ).await {
            select::Either::First(_) => {
                if current == 3 {
                    current = 0;
                } else {
                    current += 1;
                }
            }
            select::Either::Second(_) => {
                if d[current] == 9 {
                    d[current] = 0;
                } else {
                    d[current] += 1;
                }
            }
        }

        pinout = 0;
        pinout |= (map[usize::from(d[0])] as u32) << 24;
        pinout |= (map[usize::from(d[1])] as u32) << 16;
        pinout |= (map[usize::from(d[2])] as u32) << 8;
        pinout |= (map[usize::from(d[3])] as u32) << 0;
        pinout |= 1 << (24 - current * 8);

        sm0.tx().wait_push(pinout).await;

        Timer::after_millis(200).await;
    }
}
