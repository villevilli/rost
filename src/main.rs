#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rost::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rost::println;
use rost::colorchg;
use rost::Color::*;
use rost::interrupts::TICKS;
use rost::vga_driver;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    colorchg(White, LightRed);
    println!("{}", _info);
    rost::hlt_loop()
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

    colorchg(Green, Black);
    println!("Didnt crashus");

    /*vga_driver::Writer::set_cursor_pos( vga_driver::CursorPosition{
        x: 12,
        y:6
    });*/

    rost::pc_speaker::play_sound(1000);

    panic!("testpanic");

    rost::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}