#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mythic_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mythic_os::{command, print, println};


#[no_mangle]
pub extern "C" fn _start() -> ! {

    mythic_os::init();

   _main();


   #[cfg(test)]
    test_main();
    
    mythic_os::hlt_loop()
}


fn _main(){
    println!("Hello World");
    command::start_command();

}




#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    println!("{}", info);
    mythic_os::hlt_loop()
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    mythic_os::test_panic_handler(info);
}