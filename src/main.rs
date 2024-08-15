#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mythic_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mythic_os::println;


#[no_mangle]
pub extern "C" fn _start() -> ! {

    mythic_os::init();

    fn stack_overflow(){
        stack_overflow();
    }
    stack_overflow();

   _main();

   #[cfg(test)]
    test_main();
    
   loop {}
}


fn _main(){
    println!("Hola");
    println!("Mundo");
}



#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    println!("{}", info);
    loop {}
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    mythic_os::test_panic_handler(info);
}