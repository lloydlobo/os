#![no_std] // Don't link the Rust standard library.
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic.
///
/// - The PanicInfo parameter contains the file and line where the panic happened and the optional panic message.
/// - The function should never return, so it is marked as a diverging function by returning the “never” type !.
/// - There is not much we can do in this function for now, so we just loop indefinitely.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello, world!";

/// - We disable name mangling to ensure that the Rust compiler really outputs a function with the name `_start`.
/// - Name of function is _start as this is the default entry point name for most systems.
/// - The ! return type means that the function is diverging, i.e. not allowed to ever return.
/// TODO: create a VGA buffer type that encapsulates all unsafety and ensures that it is impossible to do anything wrong from the outside.
#[no_mangle] // Prevents mangling the name of this function during compilation.
pub extern "C" fn _start() -> ! {
    // Cast the integer `0xb8000` into a raw pointer.
    // - buffer is located at address 0xb8000.
    // - each character cell consists of an ASCII byte and a color byte.
    let vga_buffer = 0xb8000 as *mut u8;

    for (count, &byte) in HELLO.iter().enumerate() {
        // Use `unsafe` block because `Rust` compiler can’t prove that the raw pointers we create are valid.
        //
        // Turning off Rust’s safety checks allows you to do [five additional things](https://doc.rust-lang.org/stable/book/ch19-01-unsafe-rust.html#unsafe-superpowers).
        // - Dereference a raw pointer
        // - Call an unsafe function or method
        // - Access or modify a mutable static variable
        // - Implement an unsafe trait
        // - Access fields of unions
        unsafe {
            // Write the string byte and the corresponding color byte. `(oxb is a light cyan)`.
            //
            // `fn offset(self, count: isize)` calculates the offset from a pointer.
            // - `count` is in units of T; e.g., a `count` of 3 represents a pointer offset of `3 * size_of::<T>()` bytes.
            *vga_buffer.offset(count as isize * 2) = byte;
            *vga_buffer.offset(count as isize * 2 + 1) = 0xb;
        }
    }

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
