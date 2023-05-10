#![no_std] // Don't link the Rust standard library.
#![no_main]

use core::panic::PanicInfo;

/// - We disable name mangling to ensure that the Rust compiler really outputs a function with the name `_start`.
/// - Name of function is _start as this is the default entry point name for most systems.
/// - The ! return type means that the function is diverging, i.e. not allowed to ever return.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

/// This function is called on panic.
///
/// The PanicInfo parameter contains the file and line where the panic happened and the optional
/// panic message.
///
/// The function should never return, so it is marked as a diverging function by returning the
/// “never” type !.
///
/// There is not much we can do in this function for now, so we just loop indefinitely.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// In a typical Rust binary that links the standard library, execution starts in a C runtime
// library called crt0 (“C runtime zero”), which sets up the environment for a C application.
//
// Our freestanding executable does not have access to the Rust runtime and crt0, so we need to
// define our own entry point. Implementing the start language item wouldn’t help, since it would
// still require crt0. Instead, we need to overwrite the crt0 entry point directly.
//
// # Building for a Bare Metal target
//
// Rust uses a string called target triple.
//
// ```shell
// $ rustc --version --verbose
//
// rustc 1.71.0-nightly (1c42cb4ef 2023-04-26)
// binary: rustc
// commit-hash: 1c42cb4ef0544fbfaa500216e53382d6b079c001
// commit-date: 2023-04-26
// host: x86_64-unknown-linux-gnu
// release: 1.71.0-nightly
// LLVM version: 16.0.2
// ```
//
// By compiling for our host triple, the Rust compiler and the linker assume that there is an
// underlying operating system such as Linux or Windows that uses the C runtime by default, which
// causes the linker errors. So, to avoid the linker errors, we can compile for a different
// environment with no underlying operating system.
//
// ```shell
// $ rustup target add thumbv7em-none-eabihf
//
// info: downloading component 'rust-std' for 'thumbv7em-none-eabihf'
// info: installing component 'rust-std' for 'thumbv7em-none-eabihf'
// ```
//
// This downloads a copy of the standard (and core) library for the system. Now we can build our
// freestanding executable for this target:
//
// To build this binary, we need to compile for a bare metal target such as thumbv7em-none-eabihf:
//
// ```shell
// $ cargo build --target thumbv7em-none-eabihf
// ```
//
// Alternatively, we can compile it for the host system by passing additional linker arguments:
//
// ```shell
// # Linux
// cargo rustc -- -C link-arg=-nostartfiles
// # Windows
// cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"
// # macOS
// cargo rustc -- -C link-args="-e __start -static -nostartfiles"
// ```
//
