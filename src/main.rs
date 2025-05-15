#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
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

    loop {
        blink_all(&mut led1, &mut led2, &mut led3).await;
        beep(&mut buzzer).await;
    }
}
