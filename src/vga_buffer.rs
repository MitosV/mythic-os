use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static!{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer{
        line: false,
        line_interval: 0,
        selected_pos: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe {
            &mut *(0xb8000 as *mut Buffer)
        },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments){
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color{
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
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode{
    fn new(foreground: Color, background: Color) -> Self{
        ColorCode((background as u8) << 4 | (foreground as u8)) 
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar{
    ascii_char: u8,
    color: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer{
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}


pub struct Writer{
    line: bool,
    line_interval: usize,
    selected_pos: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {

    pub fn write_string(&mut self, s: &str){
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe)
            }
        }
    }
    
    pub fn write_byte(&mut self, byte: u8){
        self.line_interval = 3;
        self.delete_line();
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH{
                    self.new_line();
                }
                
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color = self.color_code;
                self.buffer.chars[row][col].write( ScreenChar{
                    ascii_char: byte,
                    color
                });
                self.column_position += 1;
            }
        }
        self.draw_line(true);
    }

   
    pub fn delete_last_char(&mut self){
        self.line_interval = 3;
        self.delete_line();
        if self.column_position <= 2{
            self.column_position = 2;
            return;
        }
        
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position - 1;
        let color = self.color_code;
        self.buffer.chars[row][col].write( ScreenChar{
            ascii_char: b' ',
            color
        });
        self.column_position -= 1;
        self.draw_line(true);
    }

    pub fn update_line(&mut self){
        if self.line_interval >= 10 {
            if self.line{
                self.line = false;
                self.delete_line();
            }else{
                self.line = true;
                self.draw_line(false);
            }   
            self.line_interval = 0;
        }else{
            self.line_interval += 1;
        }
    }

    fn draw_line(&mut self, force: bool){
        if !self.line && !force{
            return;
        }
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        self.selected_pos = col;
        let color = self.color_code;
        self.buffer.chars[row][col].write( ScreenChar{
            ascii_char: 179,
            color
        });
    }

    fn delete_line(&mut self){
        if self.selected_pos == 0{
            return;
        }
        let row = BUFFER_HEIGHT - 1;
        let color = self.color_code;
        self.buffer.chars[row][self.selected_pos].write( ScreenChar{
            ascii_char: 0,
            color
        });
    }

    pub fn get_last_line(&mut self, buf: &mut [u8; BUFFER_WIDTH]){
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        for col in 0..col{
            buf[col] = self.buffer.chars[row][col].read().ascii_char;
        }
    }

    pub fn new_line(&mut self){
        self.delete_line();
        for row in 1..BUFFER_HEIGHT{
            for col in 0..BUFFER_WIDTH{
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character)
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
        self.draw_line(true);
    }

    fn prev_line(&mut self){
        for row in 1..BUFFER_HEIGHT{
            for col in 0..BUFFER_WIDTH{
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row + 1][col].write(character)
            }
        }
        //self.clear_row(1);
        self.column_position = BUFFER_WIDTH - 1;
    }

    fn clear_row(&mut self, row: usize){
        let blank = ScreenChar{
            ascii_char: 0,
            color: ColorCode::new(Color::Black, Color::Black),
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank)
        }
    }
}

impl fmt::Write for Writer {

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
    
}

#[test_case]
fn test_println_output() {
    let s = "Single line test with println by char";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_char), c);
    }
}
