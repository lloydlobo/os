/// NOTE: Temporary function to write some characters to the screen.
pub fn print_something() {
    // Create a new Writer pointing to the VGAbuffer at `0xb8000`.
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        // - Cast the integer `0xb8000` as a mutable `raw pointer`.
        // - Convert it to a mutable reference by derefrencing it (through `*`)
        //   and immediately borrowing it again (through &mut).
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    // Write the byte `b'H'` to it. The `b` prefix creates a `byte literal` which represents an
    // ASCII character.
    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld");
    writer.write_byte(b'!');

    // To see the output, call `print_something` function from our `_start` function in
    // `src/main.rs`.
}

/* REGION_START: COLORS */

/// [`Color`] is a C-like enum here to explicitly specify the number for each color. Because
///
// By deriving the Copy, Clone, Debug, PartialEq, and Eq traits, we enable copy semantics for the
// type and make it printable and comparable.
//
// We use a C-like enum here to explicitly specify the number for each color. Because of the
// repr(u8) attribute, each enum variant is stored as a u8. Actually 4 bits would be sufficient,
// but Rust doesn’t have a u4 type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// The [`ColorCode`] struct contains the full color byte, containing foreground and background
/// color.
///
/// To ensure that the ColorCode has the exact same data layout as a u8, we use the
/// repr(transparent) attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/* REGION_END: COLORS */

/* REGION_START: TEXT BUFFER */

/// [`ScreenChar`] represents a screen character.
///
// Since the field ordering in default structs is undefined in Rust, we need the repr(C) attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// The [`Buffer`] struct represents the text buffer.
///
// repr(transparent) again to ensure that it has the same memory layout as its single field.
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// The [`Writer`] struct writes to the screen.
///
/// - The writer will always write to the last line and shift lines up when a line is full (or on \n).
/// - The column_position field keeps track of the current position in the last row.
/// - The current foreground and background colors are specified by color_code and a reference to
///   the VGA buffer is stored in buffer.
///
// Note that we need an explicit lifetime here to tell the compiler how long the reference is
// valid. The 'static lifetime specifies that the reference is valid for the whole program run time
// (which is true for the VGA text buffer).
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

/* REGION_END: TEXT BUFFER */

/* REGION_START: PRINTING */

/// Use [`Writer`] to modify the buffer's characters.
impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        todo!()
    }

    /// Print whole strings by converting them to bytes and print them one-by-one.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline.
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of printable ASCII range.
                _ => self.write_byte(0xfe),
            }
        }
    }
}

// The VGA text buffer only supports ASCII and the additional bytes of code page 437. Rust strings
// are UTF-8 by default, so they might contain bytes that are not supported by the VGA text buffer.
// We use a match to differentiate printable ASCII bytes (a newline or anything in between a space
// character and a ~ character) and unprintable bytes. For unprintable bytes, we print a ■
// character, which has the hex code 0xfe on the VGA hardware.

/* REGION_END: PRINTING */
