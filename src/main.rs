#![no_std] // Don't link the Rust standard library.
#![no_main]
#![feature(custom_test_frameworks)] // The `custom_test_frameworks` feature allows the use of `#[test_case]` and `#![test_runner]`.
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Turning off Rust’s safety checks allows you to do [five additional things](https://doc.rust-lang.org/stable/book/ch19-01-unsafe-rust.html#unsafe-superpowers).
// - Dereference a raw pointer
// - Call an unsafe function or method
// - Access or modify a mutable static variable
// - Implement an unsafe trait
// - Access fields of unions

use core::panic::PanicInfo;

mod vga_buffer;

// static HELLO: &[u8] = b"Hello, world!";

/// - We disable name mangling to ensure that the Rust compiler really outputs a function with the
///   name `_start`.
/// - Name of function is _start as this is the default entry point name for most
///   systems.
/// - The ! return type means that the function is diverging, i.e. not allowed to ever return.
///
/// TODO: create a VGA buffer type that encapsulates all unsafety and ensures that it is impossible to do anything wrong from the outside.
#[no_mangle] // Prevents mangling the name of this function during compilation.
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!"); // panic!("Some panic message");

    #[cfg(test)]
    test_main();

    loop {}

    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again").unwrap(); // vga_buffer::print_something();
    // write!(vga_buffer::WRITER.lock(), " some numbers: {} {}", 42, 1.337).unwrap();
    // loop {}

    // // Cast the integer `0xb8000` into a raw pointer. buffer is located at address 0xb8000.
    // // Each character cell consists of an ASCII byte and a color byte.
    // let vga_buffer = 0xb8000 as *mut u8;
    // for (count, &byte) in HELLO.iter().enumerate() { // Write the string byte and the corresponding color byte. `(oxb is a light cyan)`.
    //     unsafe { *vga_buffer.offset(count as isize * 2) = byte;
    //         *vga_buffer.offset(count as isize * 2 + 1) = 0xb; } // Use `unsafe` block because `Rust` compiler can’t prove that the raw pointers we create are valid.
    // } loop {}
}

/// This function is called on panic.
///
/// - The PanicInfo parameter contains the file and line where the panic happened and the optional panic message.
/// - The function should never return, so it is marked as a diverging function by returning the “ never” type !.
/// - There is not much we can do in this function for now, so we just loop indefinitely.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

/// Our runner just prints a short debug message and then calls each test function in the list.
///
/// The argument type &[&dyn Fn()] is a slice of trait object references of the Fn() trait. It is basically
/// a list of references to types that can be called like a function. Since the function is useless
/// for non-test runs, we use the #[cfg(test)] attribute to include it only for tests.
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1 + 1, 2);
    println!("[ok]");
}

/// To specify the exit status, we create a [`QemuExitCode`] enum. The idea is to exit with the success
/// exit code if all tests succeeded and with the failure exit code otherwise. The enum is marked as
/// #[repr(u32)] to represent each variant by a u32 integer. We use the exit code 0x10 for success and
/// 0x11 for failure.
///
/// # Usage Example
///
/// The actual exit codes don’t matter much, as long as they don’t clash with the default exit codes
/// of QEMU. For example, using exit code 0 for success is not a good idea because it becomes (0 <<
/// 1) | 1 = 1 after the transformation, which is the default exit code when QEMU fails to run. So we
/// could not differentiate a QEMU error from a successful test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// The function creates a new `Port` at 0xf4, which is the iobase of the isa-debug-exit device. Then
/// it writes the passed exit code to the port. We use u32 because we specified the iosize of the isa
/// -debug-exit device as 4 bytes. Both operations are unsafe because writing to an I/O port can generally
/// result in arbitrary behavior.
///
// Note: The problem is that cargo test considers all error codes other than 0 as failure.
// To work around this, bootimage provides a test-success-exit-code configuration key that maps a specified
// exit code to the exit code 0:
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

///////////////////////////////////////////////////

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
// # [🔗 Running our Kernel](https://os.phil-opp.com/minimal-rust-kernel/#running-our-kernel)
//
// Now that we have an executable that does something perceptible, it is time to run it. First, we
// need to turn our compiled kernel into a bootable disk image by linking it with a bootloader.
// Then we can run the disk image in the QEMU virtual machine or boot it on real hardware using a
// USB stick.
//
// ## Creating a Bootimage
//
// Turn compiled kernel into a bootable disk image: Link it with a bootloader.
//
// bootloader is responsible for initializing the CPU and loading our kernel.
//
// ## [🔗 Booting it in QEMU](https://os.phil-opp.com/minimal-rust-kernel/#booting-it-in-qemu)
//
// We can now boot the disk image in a virtual machine. To boot it in QEMU, execute the following
// command:
//
// ```shell
// $ qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin
// ```
// General form of a QEMU command line can be expressed as:
//
// $ qemu-system-x86_64 [machine opts] \
//                 [cpu opts] \
//                 [accelerator opts] \
//                 [device opts] \
//                 [backend opts] \
//                 [interface opts] \
//                 [boot opts]
//
// ### Downloads: [QEMU](https://www.qemu.org/download/)
//
// - Linux
//
// QEMU is packaged by most Linux distributions:
//     Arch: pacman -S qemu
//     Debian/Ubuntu: apt-get install qemu
//     Fedora: dnf install @virtualization
//     Gentoo: emerge --ask app-emulation/qemu
//     RHEL/CentOS: yum install qemu-kvm
//     SUSE: zypper install qemu
//
// # VGA Text Mode [🔗 Summary](https://os.phil-opp.com/vga-text-mode/#summary)
//
// In this post, we learned about the structure of the VGA text buffer and how it can be written
// through the memory mapping at address 0xb8000. We created a Rust module that encapsulates the
// unsafety of writing to this memory-mapped buffer and presents a safe and convenient interface to
// the outside.
//
// Thanks to cargo, we also saw how easy it is to add dependencies on third-party libraries. The two
// dependencies that we added, lazy_static and spin, are very useful in OS development and we will use
// them in more places in future posts.
//
// # [Testing](https://os.phil-opp.com/testing/)
//
// ## Custom test frameworks
//
// - Rust supports replacing the default test framework through the unstable custom_test_frameworks
// feature.
// - To implement a custom test framework for our kernel, we add the following to our main.rs:
//
// // in src/main.rs
//
// #![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
//
// #[cfg(test)]
// fn test_runner(tests: &[&dyn Fn()]) {
//     println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//     }
// }
//
// Our runner just prints a short debug message and then calls each test function in the list. The argument
// type &[&dyn Fn()] is a slice of trait object references of the Fn() trait. It is basically a list
// of references to types that can be called like a function. Since the function is useless for non-test
// runs, we use the #[cfg(test)] attribute to include it only for tests.
//
// - After executing the tests, our test_runner returns to the test_main function, which in turn returns
// to our _start entry point function. At the end of _start, we enter an endless loop because the entry
// point function is not allowed to return. This is a problem, because we want cargo test to exit after
// running all tests.
//
// ## Exiting QEMU
//
// Right now, we have an endless loop at the end of our _start function and need to close QEMU manually
// on each execution of cargo test. This is unfortunate because we also want to run cargo test in scripts
// without user interaction. The clean solution to this would be to implement a proper way to shutdown
// our OS. Unfortunately, this is relatively complex because it requires implementing support for either
// the APM or ACPI power management standard.
//
// Luckily, there is an escape hatch: QEMU supports a special isa-debug-exit device, which provides
// an easy way to exit QEMU from the guest system. To enable it, we need to pass a -device argument
// to QEMU. We can do so by adding a package.metadata.bootimage.test-args configuration key in our Cargo.toml:
//
// # in Cargo.toml
//
// [package.metadata.bootimage]
// test-args = ["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
//
// The bootimage runner appends the test-args to the default QEMU command for all test executables. For
// a normal cargo run, the arguments are ignored.
//
// Together with the device name (isa-debug-exit), we pass the two parameters iobase and iosize that
// specify the I/O port through which the device can be reached from our kernel.
