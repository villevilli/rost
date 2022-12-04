#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rost::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rost::colorchg;
use rost::interrupts::TICKS;
use rost::io::INPUTBUFFER;
use rost::print;
use rost::println;
use rost::serial_println;
use rost::vga_driver;
use rost::vga_driver::code_page_737_definitions::Symbols::*;
use rost::Color::*;

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
    //runs tests if built in test mode
    #[cfg(test)]
    test_main();

    colorchg(Green, Black);
    println!("Didnt crashus");

    vga_driver::WRITER
        .lock()
        .draw_symbol(Point, vga_driver::Point(10, 10));

    //rost::pc_speaker::play_sound(1000);
    loop {
        x86_64::instructions::hlt();
        let current_tick = *TICKS.read();
        let mut io = INPUTBUFFER.write();

        if io.unread_char_count != 0 {
            use pc_keyboard::DecodedKey;

            for i in &io.buffer[..io.unread_char_count] {
                match i {
                    DecodedKey::Unicode(char) => print!("{}", char),
                    DecodedKey::RawKey(raw_key) => print!("{:#?}", raw_key),
                }
            }
            io.unread_char_count = 0;
        }
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
