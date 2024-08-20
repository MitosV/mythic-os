use x86_64::instructions::port::Port;

use crate::{exit_qemu, print, println, vga_buffer::{self, BUFFER_WIDTH}};

const SHUTDOWN_PORT: u16 = 0x604;

static COMMANDS: [Command; 3] = [
    Command::new("test", || {
        println!("Simple Test Command");
    }),
    Command::new("hello", ||{
        println!("Hello, how are you?")
    }),
    Command::new("exit", ||{
        exit_qemu(crate::QemuExitCode::Success);
        unsafe {
            let mut port = Port::new(SHUTDOWN_PORT);
            port.write(0x2000u16);
        }
    }),
];


#[derive(Clone, Copy, Debug)]
struct Command{
    name: &'static str,
    function: fn(),
}


impl Command {
    const fn new(name: &'static str, function: fn()) -> Self{
        Self{
            name,
            function
        }
    }

    pub fn execute(&self){
        (self.function)();
    }
}


fn execute(name: &str){
    let mut has_command = false;
    for command in COMMANDS.iter(){
        if command.name == name {
            has_command = true;
            command.execute();
            break;
        }
    }
    if !has_command {
        println!("ERROR: dont exist the command with name '{}'", name)
    }
    print!("> ");
}

pub fn start_command(){
    print!("> ");
}

pub fn check_command(){
    let mut buf: [u8; BUFFER_WIDTH] = [0; BUFFER_WIDTH];
    let mut writer = vga_buffer::WRITER.lock();
    writer.get_last_line(&mut buf);
    if let Some(s) = buffer_to_str_slice(&mut buf){
        writer.new_line();
        drop(writer);
        execute(s);
    }
}


fn buffer_to_str_slice(buffer: &[u8; BUFFER_WIDTH]) -> Option<&str> {
    let mut end = BUFFER_WIDTH;
    while end > 0 && buffer[end - 1] == 0 || buffer[end - 1] == b' ' {
        end -= 1;
    }

    let remove_start = buffer[0] == b'>';

    // Asegúrate de que el contenido restante sea UTF-8 válido.
    let mut is_valid_utf8 = true;
    for &byte in &buffer[..end] {
        if byte > 127 {
            is_valid_utf8 = false;
            break;
        }
    }

    if is_valid_utf8 {
        let str_slice = unsafe { core::str::from_utf8_unchecked(&buffer[if remove_start{2} else {0}..end]) };
        Some(str_slice)
    } else {
        None
    }
}