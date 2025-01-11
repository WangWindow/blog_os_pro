use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    ///
    /// 一个全局的`Writer`实例，可以用于打印到VGA文本缓冲区。
    ///
    /// 由`print!`和`println!`宏使用。
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// The standard color palette in VGA text mode.
///
/// VGA文本模式中的标准颜色调色板。
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

/// A combination of a foreground and a background color.
///
/// 前景色和背景色的组合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    ///
    /// 使用给定的前景色和背景色创建一个新的`ColorCode`。
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
///
/// VGA文本缓冲区中的屏幕字符，由ASCII字符和`ColorCode`组成。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the text buffer (normally 25 lines).
///
/// 文本缓冲区的高度（通常为25行）。
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
///
/// 文本缓冲区的宽度（通常为80列）。
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
///
/// 表示VGA文本缓冲区的结构。
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
///
/// 一种写入器类型，允许将ASCII字节和字符串写入底层的`Buffer`。
///
/// 在`BUFFER_WIDTH`处换行。支持换行字符并实现`core::fmt::Write`特性。
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    ///
    /// 写入一个ASCII字节到缓冲区。
    ///
    /// 在`BUFFER_WIDTH`处换行。支持`\n`换行字符。
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
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    ///
    /// 将给定的ASCII字符串写入缓冲区。
    ///
    /// 在`BUFFER_WIDTH`处换行。支持`\n`换行字符。**不**支持带有非ASCII字符的字符串，因为它们无法在VGA文本模式下打印。
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    ///
    /// 将所有行上移一行并清除最后一行。
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    ///
    /// 通过用空白字符覆盖它来清除一行。
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Sets the foreground and background color for the text written by the writer.
    /// The colors are reset after each newline.
    /// The default color is `Color::Yellow` on `Color::Black` background.
    ///
    /// 为写入的文本设置前景色和背景色。
    /// 每次换行后颜色都会重置。
    /// 默认颜色是`Color::Yellow`在`Color::Black`背景上。
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    /// Returns the current row of the cursor.
    ///
    /// 返回光标的当前行。
    pub fn get_row(&self) -> usize {
        BUFFER_HEIGHT - 1
    }

    /// Sets the column of the cursor.
    ///
    /// 设置光标的列。
    pub fn set_column(&mut self, column: usize) {
        self.column_position = column.min(BUFFER_WIDTH - 1);
    }

    /// Clears the screen and resets the cursor position.
    ///
    /// 清屏并重置光标位置。
    pub fn clear_screen(&mut self) {
        for _ in 0..BUFFER_HEIGHT {
            self.new_line();
        }
        self.set_column(0);
    }
}

impl fmt::Write for Writer {
    /// Writes a string to the VGA text buffer.
    ///
    /// 将字符串写入VGA文本缓冲区。
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
///
/// 类似于标准库中的`print!`宏，但打印到VGA文本缓冲区。
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
///
/// 类似于标准库中的`println!`宏，但打印到VGA文本缓冲区。
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
///
/// 通过全局`WRITER`实例将给定的格式化字符串打印到VGA文本缓冲区。
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
