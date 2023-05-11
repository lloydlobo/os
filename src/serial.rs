//! # serial
//!
//! The `serial` module provides a way to send data from the kernel to the host system using the 16550
//! UART interface. It allows printing to the serial interface for testing purposes, and also provides
//! macros to print to the interface, similar to the VGA buffer.
//! The `lazy_static` and `spin` crates are used to create a static writer instance.
//! The `_print` function is used to print formatted strings to the serial port, and the
//! `serial_print!` and `serial_println!` macros allow passing token trees as arguments to generate
//! formatted strings.
//! The module uses the `fmt::Write` trait to implement printing to the serial port.
//!
//! ## Printing to the Console
//!
//! - The module uses the serial port interface, which is a simple way to send data and is easy to program.
//! - The module uses the 16550 UART, which is a common UART model and compatible with most x86 chips.
//! - QEMU can redirect the bytes sent over serial to the host's standard output or a file, allowing
//!   for easy viewing of the test output.
//!
//! Also allows us to print to the serial interface instead of the VGA text buffer in our test code.
//!
//! Note that the serial_println macro lives directly under the root namespace because we used the
//! `#[macro_export]` attribute, so importing it through use crate::serial::serial_println will not work.

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort; // The uart_16550 crate contains a SerialPort struct that represents the
                            // UART registers, but we still need to construct an instance of it ourselves.

// Like with the VGA text buffer, we use lazy_static and a spinlock to create a static writer instance
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();// By using lazy_static we can ensure that the init method is called exactly once on its first use
        Mutex::new(serial_port)
    };
}

// Like the isa-debug-exit device, the UART is programmed using port I/O.

/// - This function is similar to `vga_buffer::_print` but prints the formatted string to the VGA text
///   buffer through the global `WRITER` instance.
/// - As the `SerialPort` type already implements the `fmt ::Write` trait, there's no need to provide
///   our own implementation.
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*)=>{
        $crate::serial::_print(format_args!($($arg)*));
    }
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

// - In Rust's macro system, `$arg:tt` is a syntax that matches any `token tree`. A `token tree` can be
//   a single token or a group of token trees.
// - `$($arg:tt)*` in a macro definition means the macro can accept any number of arguments, each of which
//   can be any type of `token tree`.
// - This pattern is used in `serial_print!` macro to pass token trees as arguments to the `format_args!`
//   macro for generating a formatted string. In `serial_println!` macro, `()` matches no arguments,
//   `($fmt:expr)` matches one argument, and `($fmt:expr, $($arg:tt)*)` matches one or more arguments,
//   where the first argument is the format string expression.
