use crate::{import, malloc, math::LogicLong, network::PiranhaMessage, sc_string::ScString};

pub struct LoginMessage(pub PiranhaMessage);

impl LoginMessage {
    pub fn get_account_id(&self) -> &LogicLong {
        unsafe { &**(self.0 .0.wrapping_add(48) as *const *const LogicLong) }
    }

    pub fn get_pass_token(&self) -> Option<ScString> {
        unsafe {
            let strptr = *(self.0 .0.wrapping_add(52) as *const ScString);
            (!strptr.0.is_null()).then_some(strptr)
        }
    }

    pub fn get_scrambler_seed(&self) -> i32 {
        unsafe { *(self.0 .0.wrapping_add(244) as *const i32) }
    }
}

pub struct ExtendedSetEncryptionMessage(pub PiranhaMessage);

impl ExtendedSetEncryptionMessage {
    pub fn new() -> Self {
        import!(set_encryption_message_ctor(ptr: *const u8) -> () = 0x21BAE4);
        let instance = malloc(60);
        set_encryption_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }

    pub fn set_nonce(&mut self, nonce: &[u8]) {
        let bytes = malloc(nonce.len());
        unsafe {
            std::slice::from_raw_parts_mut(bytes as *mut u8, nonce.len()).copy_from_slice(nonce);

            *(self.0 .0.wrapping_add(48) as *mut *const u8) = bytes;
            *(self.0 .0.wrapping_add(52) as *mut i32) = nonce.len() as i32;
        }
    }

    pub fn set_scrambler_method(&mut self, method: i32) {
        unsafe {
            *(self.0 .0.wrapping_add(56) as *mut i32) = method;
        }
    }
}

pub struct LoginOkMessage(pub PiranhaMessage);

impl LoginOkMessage {
    pub fn new() -> Self {
        import!(login_ok_message_ctor(ptr: *const u8) -> () = 0x2058F0);

        let instance = malloc(124);
        login_ok_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }

    pub fn set_account_id(&mut self, account_id: LogicLong) {
        unsafe {
            *(self.0 .0.wrapping_add(48) as *mut usize) = account_id.to_heap() as usize;
        }
    }

    pub fn set_home_id(&mut self, home_id: LogicLong) {
        unsafe {
            *(self.0 .0.wrapping_add(52) as *mut usize) = home_id.to_heap() as usize;
        }
    }

    pub fn set_pass_token(&mut self, pass_token: &str) {
        unsafe {
            *(self.0 .0.wrapping_add(56) as *mut usize) = ScString::from(pass_token).0 as usize;
        }
    }

    pub fn set_server_major_version(&mut self, value: i32) {
        unsafe {
            *(self.0 .0.wrapping_add(76) as *mut i32) = value;
        }
    }

    pub fn set_server_build(&mut self, value: i32) {
        unsafe {
            *(self.0 .0.wrapping_add(80) as *mut i32) = value;
        }
    }

    pub fn set_content_version(&mut self, value: i32) {
        unsafe {
            *(self.0 .0.wrapping_add(84) as *mut i32) = value;
        }
    }

    pub fn set_server_environment(&mut self, value: &str) {
        unsafe {
            *(self.0 .0.wrapping_add(88) as *mut usize) = ScString::from(value).0 as usize;
        }
    }
}

pub struct KeepAliveServerMessage(pub PiranhaMessage);

impl KeepAliveServerMessage {
    pub fn new() -> Self {
        import!(keep_alive_server_message_ctor(ptr: *const u8) -> () = 0x20390C);
        let instance = malloc(48);
        keep_alive_server_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }
}
