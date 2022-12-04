use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use spin::RwLock;

use crate::serial_println;

lazy_static! {
    pub static ref INPUTBUFFER: RwLock<InputBuffer> = RwLock::new(InputBuffer {
        buffer: [DecodedKey::Unicode('0'); 255],
        unread_char_count: 0,
    });
}

#[derive(Debug, Clone, Copy)]
pub struct InputBuffer {
    pub buffer: [DecodedKey; 255],
    pub unread_char_count: usize,
}

pub struct InputReturn(usize, [DecodedKey; 255]);

impl InputBuffer {
    ///appends to the front of the buffer deleting the last item in the buffer
    pub fn write_key(&mut self, key: DecodedKey) {
        self.buffer[..].rotate_right(1);
        self.buffer[0] = key;
        self.unread_char_count = self.unread_char_count.saturating_add(1);
    }

    ///Returns a copy of the current INPUTBUFFER and sets the read index to 0
    pub fn read(mut self) -> InputReturn {
        let return_object = InputReturn(self.unread_char_count.clone(), self.buffer.clone());
        self.unread_char_count = 0;
        return_object
    }

    pub fn read_unread(self) {
        //This can be added if we ever decide to add such a thing known as a heap
        todo!();
    }
}

#[test_case]
fn test_write_unicode() {
    {
        INPUTBUFFER.write().write_key(DecodedKey::Unicode('e'));
    }
    let inputbuffer = INPUTBUFFER.read();
    assert_eq!(inputbuffer.buffer[0], DecodedKey::Unicode('e'));
}

#[test_case]
fn test_write_rawkey() {
    {
        INPUTBUFFER
            .write()
            .write_key(DecodedKey::RawKey(pc_keyboard::KeyCode::Delete))
    }

    let inputbuffer = INPUTBUFFER.read();
    assert_eq!(
        inputbuffer.buffer[0],
        DecodedKey::RawKey(pc_keyboard::KeyCode::Delete)
    );
}

#[test_case]
fn test_read_index() {
    for i in 2..6 {
        INPUTBUFFER.write().write_key(DecodedKey::Unicode(i.into()));
    }
    let input_buffer = INPUTBUFFER.read();
    match input_buffer.buffer[input_buffer.unread_char_count - 1] {
        DecodedKey::Unicode(character) => serial_println!("{}", character),
        DecodedKey::RawKey(key) => serial_println!("{:?}", key),
    }
    assert_eq!(input_buffer.unread_char_count, 4);
}
