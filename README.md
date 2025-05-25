# Morse Coder

**Morse Coder** is a Rust-based embedded system that converts typed text into Morse code using a Raspberry Pi Pico 2W. 
The user enters text through a 4x4 matrix keypad, which is then translated into Morse code and transmitted via:

* Sound signals (buzzer)
* Light signals (LEDs)
* Visual display (LCD screen using dots and dashes)

The goal is to provide a compact and intuitive system for encoding and visualizing Morse messages using embedded Rust.  
The project required around **60 hours** of hands-on development, from initial hardware setup to final software testing and documentation.

## Functionality Description

The user types a message using a 4x4 matrix keypad with multitap logic. The system translates each confirmed 
character into Morse code. The Morse code output is:
* Played through a buzzer (short/long beeps)
* Visualized with LEDs (dot = 1 LED, dash = 3 LEDs)
* Shown on an I2C LCD screen using dots (.) and dashes (-)

Additional functionality includes:

Additional features:
- Predefined commands to send **HELLO** and **SOS**
- Displaying **fun facts** on screen
- A **Morse code quiz mode**

---

## Requirments

### Hardware

| Component | Purpose | Function |
|:----------|:--------|:---------|
| **Raspberry Pi Pico 2W** | Main controller of the system | Reads input text, processes it, and controls outputs to buzzer, LEDs, and LCD |
| **4x4 Matrix Keypad** | Provides text input | Acts as the input device for entering characters |
| **Active Buzzer** | Outputs Morse code through sound | Emits short and long beeps representing dots and dashes |
| **LEDs** x 3 | Visual representation of Morse code signals | - When a dot (.) is detected, only **one LED** lights up (the middle one).<br/>- When a dash (_) is detected, **all three LEDs** light up simultaneously. |
| **LCD Display** | Displays the Morse code translation | Shows real-time dot and dash output |
| **Breadboard + Jumper Wires** | Temporary prototyping connections | Connects components to the Raspberry Pi Pico during development |

### Software

- **Rust** with `thumbv6m-none-eabi` target
- **Embassy** async embedded framework
- `probe-rs` or `cargo-embed` for flashing code
- See dependencies in the table below

---

## Software Dependencies

| Library | Description | Usage in your code |
|:--------|:------------|:-------------------|
| [embassy-rp](https://github.com/embassy-rs/embassy) | HAL for Raspberry Pi Pico W | Used for I2C interface, peripheral initialization |
| [embassy-executor](https://github.com/embassy-rs/embassy) | Asynchronous task runtime | Used for `#[embassy_executor::main]` and async tasks |
| [embassy-time](https://github.com/embassy-rs/embassy) | Asynchronous timers and delays | Used for `Timer::after()` non-blocking delays |
| [lcd1602_driver](https://crates.io/crates/lcd1602_driver) | Driver for controlling LCD1602 over I2C | Used for initializing and writing text to the LCD |
| [defmt](https://github.com/knurling-rs/defmt) | Lightweight embedded logging | Used for debug prints (`defmt::info!`) |
| [defmt-rtt](https://github.com/knurling-rs/defmt) | RTT transport for `defmt` | Sends logs to the host |
| [panic-probe](https://github.com/knurling-rs/defmt) | Panic handler for embedded targets | Handles panics and sends diagnostic info |
| [embedded-hal](https://github.com/rust-embedded/embedded-hal) | Traits for I2C, GPIO and delays | Used indirectly via `embassy-rp` and `lcd1602_driver` |
| [heapless](https://crates.io/crates/heapless) | Fixed-size data structures for no_std | Used for buffer storage (messages, Morse code) |
| [rand](https://crates.io/crates/rand) + `small_rng` | Random number generation | Used for quiz feature (random letter) |

## Hardware & Software Design

### System Arhitecture

The following diagram shows the system architecture:

![System Architecture Diagram](./Images/diagram.png)

The **Raspberry Pi Pico 2W** acts as the brain of the system, coordinating all interactions between input and output components.

- The **4x4 matrix keypad** provides the user input.
- The **Pico** processes the text and converts it into Morse code.
- The **LEDs** and **buzzer** output the corresponding Morse signals.
- The **LCD screen** displays the Morse code in real time using dots (.) and dashes (_).

### Keypad Interface Design

![Keypad Diagram](./Images/keypad.jpg)

| Key | Function |
|-----|----------|
| `HELLO` | Sends `HELLO` in Morse |
| `S.O.S.` | Sends SOS sequence |
| `DEMO` | Quiz mode (guess the letter) |
| `FUN FACTS` | Shows trivia |
| `MODE` | Switches text/number input |
| `TEST ALL` | Sends full message buffer |

---

### Software Flow

![Software Flow Diagram](./Images/software_flow.webp)

The software operates in a continuous loop, monitoring keypad input. When a key is pressed, the program first checks whether it is a special key (such as HELLO, S.O.S., FUN FACTS, DEMO, or TEST ALL). If so, it executes the corresponding function: displaying or transmitting predefined Morse code sequences, showing a fun fact, playing a Morse quiz, or sending the entire message typed so far. If the key is not a special command, the program proceeds to check the current input mode—Text or Numeric. In Text mode, multitap logic is used to determine the intended character, while in Numeric mode digits are added directly. After a one-second pause without further taps, the current character is confirmed, added to a message buffer, and its Morse code is displayed and played. The system then returns to listening for the next key input.


## Acknowledgements

- Based on coursework from the Microprocessor Architecture course @ Politehnica University of Bucharest
- Built using [Embassy](https://embassy.dev/) async embedded framework