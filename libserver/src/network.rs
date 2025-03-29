use tracing::info;

use crate::{import, malloc, sc_string::ScString};

pub struct Messaging(*const u8);

#[repr(C)]
pub struct Connection {
    pub fd: i32,
    pub is_connected: bool,
    unk_1: i32,
    unk_2: i32,
    unk_3: i32,
}

impl Messaging {
    pub fn new(fd: i32) -> Self {
        import!(messaging_ctor(ptr: *const u8, queue_size: i32) -> () = 0x224D8E);

        let instance = malloc(300);
        messaging_ctor(instance, 50);

        let mut messaging = Self(instance);
        messaging.get_connection().fd = fd;
        messaging.get_connection().is_connected = true;

        messaging
    }

    pub fn set_encrypters(&mut self, encrypter: RC4Encrypter, decrypter: RC4Encrypter) {
        import!(messaging_set_encrypters(ptr: *const u8, en: *const u8, de: *const u8, a4: i32) -> () = 0x224C54);
        messaging_set_encrypters(self.0, encrypter.0, decrypter.0, 0);
    }

    pub fn set_message_factory(&mut self, factory: *const LogicMagicMessageFactory) {
        unsafe { *(self.0.wrapping_add(4) as *mut usize) = factory as usize }
    }

    pub fn on_receive(&mut self) {
        import!(messaging_on_receive(ptr: *const u8, connection: *mut Connection) -> () = 0x225CE6);
        unsafe { messaging_on_receive(self.0, std::mem::transmute(self.0.wrapping_add(64))) }
    }

    pub fn next_message(&mut self) -> Option<PiranhaMessage> {
        import!(messaging_next_message(ptr: *const u8) -> usize = 0x22529E);

        let message = messaging_next_message(self.0);
        (message != 0).then_some(PiranhaMessage(message as *const u8))
    }

    pub fn send(&mut self, message: PiranhaMessage) {
        import!(messaging_send(ptr: *const u8, message: *const u8) -> () = 0x225492);
        info!(
            "Messaging::send: sending message of type {}",
            message.get_message_type()
        );

        messaging_send(self.0, message.0);
    }

    pub fn on_wakeup(&mut self) {
        import!(messaging_on_wakeup(ptr: *const u8, connection: *mut Connection) -> () = 0x225118);
        unsafe { messaging_on_wakeup(self.0, std::mem::transmute(self.0.wrapping_add(64))) }
    }

    pub fn get_connection(&mut self) -> &mut Connection {
        unsafe { std::mem::transmute(self.0.wrapping_add(64)) }
    }

    pub fn scramble_nonce_using_mersenne_twister(seed: i32, nonce: &mut [u8]) {
        import!(messaging_scramble_nonce_using_mersenne_twister(seed: i32, nonce: *const u8, nonce_len: i32) -> () = 0x2710FE);
        messaging_scramble_nonce_using_mersenne_twister(seed, nonce.as_ptr(), nonce.len() as i32);
    }
}

#[repr(transparent)]
pub struct PiranhaMessage(pub *const u8);

impl PiranhaMessage {
    pub fn get_message_type(&self) -> u16 {
        unsafe {
            let fn_ptr = ((*(self.0 as *const usize)) + 20) as *const usize;
            std::mem::transmute::<_, extern "C" fn(*const u8) -> u16>(*fn_ptr)(self.0)
        }
    }
}

#[repr(C)]
pub struct LogicMagicMessageFactory {
    vtable: usize,
}

impl LogicMagicMessageFactory {
    pub const RC4_KEY: &str = "fhsd6f86f67rt8fw78fw789we78r9789wer6re";

    pub fn new() -> Self {
        import!(logic_magic_message_factory_ctor(ptr: *mut u8) -> () = 0x1DBD3E);

        let mut instance = Self { vtable: 0 };
        unsafe {
            logic_magic_message_factory_ctor(std::mem::transmute(&mut instance));
        }

        instance
    }
}

#[repr(transparent)]
pub struct RC4Encrypter(*const u8);

impl RC4Encrypter {
    pub fn new(key: &str, nonce: &str) -> Self {
        import!(rc4_encrypter_ctor(ptr: *const u8, key: *const u8, nonce: *const u8) -> () = 0x243DD2);

        let instance = malloc(268);
        rc4_encrypter_ctor(instance, ScString::from(key).0, ScString::from(nonce).0);
        Self(instance)
    }

    pub fn new_with_nonce_bytes(key: &str, nonce: &[u8]) -> Self {
        import!(rc4_encrypter_ctor(ptr: *const u8, key: *const u8, nonce: *const u8, nonce_len: i32) -> () = 0x243D6E);

        let instance = malloc(268);
        let nonce_bytes = malloc(nonce.len());
        unsafe {
            std::slice::from_raw_parts_mut(nonce_bytes as *mut u8, nonce.len())
                .copy_from_slice(nonce);
        };

        rc4_encrypter_ctor(
            instance,
            ScString::from(key).0,
            nonce_bytes,
            nonce.len() as i32,
        );

        Self(instance)
    }
}
