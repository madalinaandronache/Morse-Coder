#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Input, Pull};
use embassy_rp::{init, bind_interrupts, i2c::InterruptHandler};
use embassy_rp::i2c::{I2c, Config as I2cConfig};
use embassy_rp::peripherals::I2C1;
use embassy_time::{Timer, Duration, Delay, Instant};
use heapless::String;
use {defmt_rtt as _, panic_probe as _};
use lcd1602_driver::{
    lcd::{Basic, Ext, Lcd, Config},
    sender::I2cSender,
};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[derive(Copy, Clone, PartialEq)]
enum InputMode {
    Text,
    Numeric,
}


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

// Initialize the keypad
fn init_keypad(
    p6: embassy_rp::peripherals::PIN_6,
    p7: embassy_rp::peripherals::PIN_7,
    p8: embassy_rp::peripherals::PIN_8,
    p9: embassy_rp::peripherals::PIN_9,
    p10: embassy_rp::peripherals::PIN_10,
    p11: embassy_rp::peripherals::PIN_11,
    p12: embassy_rp::peripherals::PIN_12,
    p13: embassy_rp::peripherals::PIN_13,
) -> ([Input<'static>; 4], [Output<'static>; 4], [[char; 4]; 4]) {
    let rows = [
        Input::new(p6, Pull::Up),
        Input::new(p7, Pull::Up),
        Input::new(p8, Pull::Up),
        Input::new(p9, Pull::Up),
    ];

    let cols = [
        Output::new(p10, Level::High),
        Output::new(p11, Level::High),
        Output::new(p12, Level::High),
        Output::new(p13, Level::High),
    ];

    let keys = [
        ['1', '2', '3', 'A'],
        ['4', '5', '6', 'B'],
        ['7', '8', '9', 'C'],
        ['*', '0', '#', 'D'],
    ];

    (rows, cols, keys)
}

// Transformation of a character into Morse signals
fn morse_table(c: char) -> Option<&'static str> {
    match c.to_ascii_uppercase() {
        'A' => Some(".-"),
        'B' => Some("-..."),
        'C' => Some("-.-."),
        'D' => Some("-.."),
        'E' => Some("."),
        'F' => Some("..-."),
        'G' => Some("--."),
        'H' => Some("...."),
        'I' => Some(".."),
        'J' => Some(".---"),
        'K' => Some("-.-"),
        'L' => Some(".-.."),
        'M' => Some("--"),
        'N' => Some("-."),
        'O' => Some("---"),
        'P' => Some(".--."),
        'Q' => Some("--.-"),
        'R' => Some(".-."),
        'S' => Some("..."),
        'T' => Some("-"),
        'U' => Some("..-"),
        'V' => Some("...-"),
        'W' => Some(".--"),
        'X' => Some("-..-"),
        'Y' => Some("-.--"),
        'Z' => Some("--.."),
        '0' => Some("-----"),
        '1' => Some(".----"),
        '2' => Some("..---"),
        '3' => Some("...--"),
        '4' => Some("....-"),
        '5' => Some("....."),
        '6' => Some("-...."),
        '7' => Some("--..."),
        '8' => Some("---.."),
        '9' => Some("----."),
        _ => None,
    }
}


async fn display_letter_morse(
    c: char,
    led1: &mut Output<'static>,
    led2: &mut Output<'static>,
    led3: &mut Output<'static>,
    buzzer: &mut Output<'static>,
) {
    if let Some(code) = morse_table(c) {
        for symbol in code.chars() {
            match symbol {
                '.' => {
                    led2.set_high();
                    buzzer.set_high();
                    Timer::after(Duration::from_millis(200)).await;
                    led2.set_low();
                    buzzer.set_low();
                }
                '-' => {
                    led1.set_high();
                    led2.set_high();
                    led3.set_high();
                    buzzer.set_high();
                    Timer::after(Duration::from_millis(600)).await;
                    led1.set_low();
                    led2.set_low();
                    led3.set_low();
                    buzzer.set_low();
                }
                _ => {}
            }

            // Break between signals
            Timer::after(Duration::from_millis(200)).await;
        }

        // Break between letters
        Timer::after(Duration::from_millis(600)).await;
    }
}

fn get_multitap_chars(key: char) -> Option<&'static [char]> {
    match key {
        '2' => Some(&['A', 'B', 'C']),
        '3' => Some(&['D', 'E', 'F']),
        '4' => Some(&['G', 'H', 'I']),
        '5' => Some(&['J', 'K', 'L']),
        '6' => Some(&['M', 'N', 'O']),
        '7' => Some(&['P', 'Q', 'R', 'S']),
        '8' => Some(&['T', 'U', 'V']),
        '9' => Some(&['W', 'X', 'Y', 'Z']),
        '0' => Some(&[' ']),
        _ => None,
    }
}

fn confirm_key(key: char, tap_index: usize, mode: InputMode) -> Option<char> {
    match mode {
        InputMode::Numeric => {
            if key.is_ascii_digit() {
                Some(key)
            } else {
                None
            }
        }
        InputMode::Text => {
            if let Some(chars) = get_multitap_chars(key) {
                Some(chars[tap_index % chars.len()])
            } else {
                None
            }
        }
    }
}


async fn handle_multitap_input(
    rows: &mut [Input<'static>; 4],
    cols: &mut [Output<'static>; 4],
    keys: [[char; 4]; 4],
    last_key: &mut Option<char>,
    tap_index: &mut usize,
    last_press_time: &mut Instant,
    mode: InputMode,
) -> Option<(char, bool)> {
    let now = Instant::now();
    let timeout = Duration::from_millis(1000);

    // Confirm the key after timeout
    if let Some(last) = last_key {
        if now.checked_duration_since(*last_press_time).unwrap_or(timeout) >= timeout {
            if let Some(ch) = confirm_key(*last, *tap_index, mode) {
                *last_key = None;
                *tap_index = 0;
                return Some((ch, false));
            }
        }
    }

    // Detect the key pressed
    if let Some(key) = scan_keypad(rows, cols, keys).await {
        if key == '#' {
            defmt::info!("Mode switch requested via '#'");
            *last_key = None;
            *tap_index = 0;
            return Some(('#', true));
        }

        if key == '*' {
            defmt::info!("Fun Fact key pressed: '*'");
            *last_key = None;
            *tap_index = 0;
            return Some(('*', false));
        }

        if get_multitap_chars(key).is_none() {
            defmt::warn!("Unmapped key '{}'", key);
            *last_key = None;
            *tap_index = 0;
            return None;
        }

        defmt::info!("Pressed key: {}", key);

        if Some(key) == *last_key {
            *tap_index += 1;
            defmt::info!("Same key tapped {} time(s)", *tap_index + 1);
        } else {
            if let Some(last) = *last_key {
                if let Some(ch) = confirm_key(last, *tap_index, mode) {
                    *last_key = Some(key);
                    *tap_index = 0;
                    *last_press_time = now;
                    return Some((ch, false));
                }
            }
            *tap_index = 0;
        }

        *last_key = Some(key);
        *last_press_time = now;

        let chars = get_multitap_chars(key).unwrap();
        let ch = chars[*tap_index % chars.len()];
        defmt::info!("Current character: '{}'", ch);
    }

    Timer::after(Duration::from_millis(50)).await;
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

    let (mut row_pins, mut col_pins, keys) = init_keypad(
        p.PIN_6, p.PIN_7, p.PIN_8, p.PIN_9,
        p.PIN_10, p.PIN_11, p.PIN_12, p.PIN_13,
    );

    // Keypad multiple options
    let mut last_key: Option<char> = None;
    let mut tap_index: usize = 0;
    let mut last_press_time = Instant::now();
    // Keep the input mode
    let mut mode = InputMode::Text;


    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let mut i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, I2cConfig::default());
   
    let mut delay = Delay;
    let mut sender = I2cSender::new(&mut i2c, 0x27);
    let mut lcd = Lcd::new(&mut sender, &mut delay, Config::default(), None);

    // Initialization message on LCD
    Timer::after(Duration::from_millis(300)).await;
    lcd.return_home();
    Timer::after(Duration::from_millis(5)).await;

    lcd.clean_display();
    Timer::after(Duration::from_millis(5)).await;
    lcd.set_cursor_pos((0, 0));
    lcd.write_str_to_cur("Keypad Ready!");

    const FUN_FACTS: &[&str] = &[
        "E is the most used letter.",
        "SOS is ...---...",
        "Morse Code was invented in 1836.",
        "The '@' in Morse is .--.-.",
        "Used in WW2 communications.",
        "SMS systems borrowed Morse ideas.",
        "The Titanic sent SOS.",
        "CQD was used before SOS.",
        "Morse sent over radio & light.",
        "NASA used Morse in beacons.",
    ];

    let mut fact_index = 0;


    loop {
        if let Some((c, is_mode_switch)) = handle_multitap_input(
            &mut row_pins,
            &mut col_pins,
            keys,
            &mut last_key,
            &mut tap_index,
            &mut last_press_time,
            mode
        ).await {
            if is_mode_switch {
                mode = match mode {
                    InputMode::Text => InputMode::Numeric,
                    InputMode::Numeric => InputMode::Text,
                };

                lcd.clean_display();
                lcd.set_cursor_pos((0, 0));
                lcd.write_str_to_cur(match mode {
                    InputMode::Text => "Mode: Text",
                    InputMode::Numeric => "Mode: 123",
                });
                continue;
            }

            defmt::info!("Final confirmed input: '{}'", c);

            if c == '*' {
                let fact = FUN_FACTS[fact_index % FUN_FACTS.len()];
                fact_index += 1;

                lcd.clean_display();
                lcd.set_cursor_pos((0, 0));
                lcd.write_str_to_cur("Fun Fact:");

                let mut buffer = String::<64>::new();
                buffer.push_str(fact).ok();

                let len = buffer.len();
                let display_width = 16;

                for i in 0..=(len.saturating_sub(display_width)) {
                    lcd.set_cursor_pos((0, 1));
                    let window = &buffer[i..i + display_width];
                    lcd.write_str_to_cur(window);
                    Timer::after(Duration::from_millis(600)).await;
                }

                continue;
            }

            if let Some(code) = morse_table(c) {
                lcd.clean_display();
                lcd.set_cursor_pos((0, 0));
                lcd.write_str_to_cur("Char: ");
                lcd.write_char_to_cur(c);

                lcd.set_cursor_pos((0, 1));
                lcd.write_str_to_cur("Morse: ");
                lcd.write_str_to_cur(code);
            }

            display_letter_morse(c, &mut led1, &mut led2, &mut led3, &mut buzzer).await;
        }
    }
}
