use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::{
    instructions::port::{self, Port, ReadWriteAccess},
    structures::port::PortWrite,
};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_driver::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[allow(unused)]
pub fn colorchg(foreground_color: Color, background_color: Color) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER
            .lock()
            .change_color(foreground_color, background_color);
    });
}

pub fn change_screen_color(foreground_color: Color, background_color: Color) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER
            .lock()
            .change_screen_color(foreground_color, background_color);
    });
}

pub fn move_cursor_by(x: i8, y: i8) {
    WRITER.lock().move_cursor_by(x, y)
}

#[derive(Debug)]
pub struct CursorPosition {
    x: u8,
    y: u8,
}

//the interface for the vga text mode text buffer
// 4 bit color table (using u8 since no u4 exsists)
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

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        registers: Registers {
            crtc_address: Port::new(0x3D4),
            crtc_data: Port::new(0x3D5)
        },
    });
}
//A text character color code repsented as an u8 containing both the foreground and background color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

//A vga text mode character with a colour and character code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

//the vga text mode 2 dimensional chracter buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct Registers {
    crtc_address: Port<u8>,
    crtc_data: Port<u8>,
}

//external implementation for writing to screen
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    registers: Registers,
}

//let mut crtc_address_register = Port::new(0x3D4);
//let mut crtc_data_register = Port::new(0x3D5);

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
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    ///Writes a string to the bottom of the screen
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    ///Changes the color currently writen by the writer
    pub fn change_color(&mut self, foreground_color: Color, background_color: Color) {
        self.color_code = ColorCode::new(foreground_color, background_color);
    }

    ///Iterates over the whole screen buffer and changes the color of each symbol
    pub fn change_screen_color(&mut self, foreground_color: Color, background_color: Color) {
        self.change_color(foreground_color, background_color);

        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: char.ascii_character,
                    color_code: ColorCode::new(foreground_color, background_color),
                })
            }
        }
    }

    ///Writes to a specified crtc data bus
    fn crtc_write(&mut self, address: u8, value: u8) {
        without_interrupts(move || unsafe {
            self.registers.crtc_address.write(address);
            self.registers.crtc_data.write(value);
        });
    }

    ///Reads a specified crtc data bus and returns the value
    fn crtc_read(&mut self, address: u8) -> u8 {
        without_interrupts(move || unsafe {
            self.registers.crtc_address.write(address);
            self.registers.crtc_data.read()
        })
    }

    ///Moves the cursor x and y and wraps the cursor around if it goes over the side
    pub fn move_cursor_by(&mut self, x: i8, y: i8) {
        let mut cursor_pos = self.get_cursor_position();

        //without_interrupts(|| {
        //println!("{:#?}", cursor_pos);
        //});

        let mut new_x: i16 = (cursor_pos.x as i8 + x).into();
        let mut new_y: i16 = (cursor_pos.y as i8 + y).into();

        //without_interrupts(|| {
        //println!("new_x: {} new_y: {}", new_x, new_y);
        //});
        //BUFFER_WIDTH as i16 - new_x
        if new_x < 0 {
            new_x = BUFFER_WIDTH as i16 + new_x;
        } else if new_x > BUFFER_WIDTH as i16 + 1 {
            new_x = new_x + BUFFER_WIDTH as i16;
        }

        cursor_pos.x = new_x as u8;
        cursor_pos.y = new_y as u8;

        self.set_cursor_pos(cursor_pos)
    }

    ///Takes a Cursorposition and sets it as the current cursorposition
    pub fn set_cursor_pos(&mut self, cp: CursorPosition) {
        let pos = cp.y as u16 * BUFFER_WIDTH as u16 + cp.x as u16;

        let [high, low] = pos.to_be_bytes();

        self.crtc_write(0x0E, high);
        self.crtc_write(0x0F, low);
    }

    ///Gets the current curor position and returns a CursorPosition
    pub fn get_cursor_position(&mut self) -> CursorPosition {
        //gets the first and last 8 bits of the cursor position by reading them from the vga cards crtc addresses
        let first_bits = self.crtc_read(0x0E);
        let last_bits = self.crtc_read(0x0F);

        // connects the first and last bits into an u16
        let cursor_distance = ((first_bits as u16) << 8) | last_bits as u16;

        //performs math to extract the x and y position
        CursorPosition {
            x: (|| -> u8 {
                (cursor_distance - ((cursor_distance / 80) * 80))
                    .try_into()
                    .unwrap()
            })(),
            y: (|| -> u8 { (cursor_distance / 80).try_into().unwrap() })(),
        }
    }

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

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
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

#[test_case]
fn test_color_change() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    let s = "Some test string that fits on a single line";

    colorchg(Color::White, Color::Black);
    colorchg(Color::Green, Color::Blue);

    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");

        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
            assert_eq!(
                screen_char.color_code,
                ColorCode::new(Color::Green, Color::Blue)
            )
        }
    });
}
