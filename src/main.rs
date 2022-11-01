#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rost::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rost::println;
use rost::colorchg;
use rost::Color::*;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    
    colorchg(White, LightRed);
    println!("{}", _info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rost::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {  
    rost::init();

    println!("Hello {}", "World!");

    for i in 0..15{
        
        println!("{}", i);

    }

    #[cfg(test)]
    test_main();

    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    println!("it didnt crash!!");
    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
