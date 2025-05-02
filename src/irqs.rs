//! This module contains the struct that implements the interrupt handlers.
//!
//! # Example for editing this module:
//! ```rust,ignore
//! // I2C peripheral interrupt handler
//! use embassy_rp::i2c::InterruptHandler as I2cInterruptHandler;
//! use embassy_rp::peripherals::I2C1;
//!
//! bind_interrupts!(
//!    pub(super) struct Irqs {
//!         I2C1_IRQ => I2cInterruptHandler<I2C1>;
//!    });
//! ```

use embassy_rp::bind_interrupts;
// You can import here and alias the handlers.

bind_interrupts!(
    pub(super) struct Irqs {
        // Here you add all you bindings following the example from the module's top.
        // You can see all interrupt ids defined here: https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf
        // in the "3.2. Interrupts" section.
    }
);
