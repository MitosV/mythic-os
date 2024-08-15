#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mythic_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mythic_os::println;


#[test_case]
fn test_printing() {
    println!("test_println output");
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mythic_os::test_panic_handler(info);
}