use x86_64::instructions::port::Port;

pub fn play_sound(freq: u32){
    //todo fix
    /*let div: u32 = 1193180 / freq;

    let mut port = Port::new(0x43);
    let mut port2 = Port::new(0x43);

    unsafe{
        port.write(0xb6 as u8);
        port2.write(div);
        port2.write(div >> 8);
    }

    let tmp: u8 = unsafe{port3.read()};

    if tmp != (tmp | 3) {
        unsafe{
            port2.write(tmp | 3)
        }
    }*/
}
 
pub fn stop_sound() {
    let mut port3 = Port::new(0x61);
    let tmp: u8 = unsafe { port3.read() } & 0xFC;

    unsafe{
        port3.write(tmp)
    }
}

#[test_case]
fn test_play_sound(){
    play_sound(1000);
}


#[test_case]
fn test_stop_sound(){
    stop_sound();
}