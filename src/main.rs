#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Input, Pull};
use embassy_rp::init;
use embassy_time::{Timer, Duration};
use {defmt_rtt as _, panic_probe as _};

// Blink all 3 LEDs at the same time
async fn blink_all(
    led1: &mut Output<'static>,
    led2: &mut Output<'static>,
    led3: &mut Output<'static>,
) {
    led1.set_high();
    led2.set_high();
    led3.set_high();
    Timer::after(Duration::from_millis(500)).await;

    led1.set_low();
    led2.set_low();
    led3.set_low();
    Timer::after(Duration::from_millis(500)).await;
}

// Beep the buzzer
async fn beep(buzzer: &mut Output<'static>) {
    buzzer.set_high();
    Timer::after(Duration::from_millis(200)).await;
    buzzer.set_low();
}

// Check if a button is pressed
async fn scan_keypad(
    rows: &mut [Input<'static>; 4],
    cols: &mut [Output<'static>; 4],
    keys: [[char; 4]; 4],
) -> Option<char> {
    for (c, col) in cols.iter_mut().enumerate() {
        col.set_low();

        for (r, row) in rows.iter().enumerate() {
            if row.is_low() {
                while row.is_low() {
                    Timer::after(Duration::from_millis(10)).await;
                }

                Timer::after(Duration::from_millis(100)).await;
                col.set_high();
                return Some(keys[r][c]);
            }
        }

        col.set_high();
    }

    None
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the peripherals
    let p = init(Default::default());

    // Initialize 3 LEDs 
    let mut led1 = Output::new(p.PIN_18, Level::Low);
    let mut led2 = Output::new(p.PIN_19, Level::Low);
    let mut led3 = Output::new(p.PIN_20, Level::Low);

    // Initialize the buzzer
    let mut buzzer = Output::new(p.PIN_16, Level::Low);

    // Initialize the keypad
    let mut row_pins = [
        Input::new(p.PIN_6, Pull::Up),
        Input::new(p.PIN_7, Pull::Up),
        Input::new(p.PIN_8, Pull::Up),
        Input::new(p.PIN_9, Pull::Up),
    ];

    let mut col_pins = [
        Output::new(p.PIN_10, Level::High),
        Output::new(p.PIN_11, Level::High),
        Output::new(p.PIN_12, Level::High),
        Output::new(p.PIN_13, Level::High),
    ];

    // Standard configuration of the keypad
    let keys: [[char; 4]; 4] = [
        ['1', '2', '3', 'A'], 
        ['4', '5', '6', 'B'],
        ['7', '8', '9', 'C'],
        ['*', '0', '#', 'D'],
    ];

    blink_all(&mut led1, &mut led2, &mut led3).await;
    beep(&mut buzzer).await;

    loop {
        if let Some(key) = scan_keypad(&mut row_pins, &mut col_pins, keys).await {
            defmt::info!("Key: {}", key);
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}
