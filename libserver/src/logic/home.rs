use crate::{import, malloc, sc_string::ScString};

#[repr(transparent)]
pub struct LogicClientHome(pub *const u8);

impl LogicClientHome {
    pub fn new() -> Self {
        import!(logic_client_home_ctor(ptr: *const u8) -> () = 0x1D3CE2);

        let instance = malloc(48);
        logic_client_home_ctor(instance);
        Self(instance)
    }

    pub fn set_home_json(&mut self, home_json: &str) {
        unsafe {
            *(self.0.wrapping_add(36) as *mut usize) = ScString::from(home_json).0 as usize;
        }
    }
}
