use crate::import;

use super::{avatar::LogicAvatar, json::LogicJSONNode, time::LogicTime};

#[repr(transparent)]
pub struct LogicLevel(pub *const u8);

impl LogicLevel {
    pub fn get_time(&self) -> &LogicTime {
        unsafe { &*(self.0.wrapping_add(88) as *const LogicTime) }
    }

    pub fn get_home_owner_avatar<T: LogicAvatar>(&self) -> Option<T> {
        unsafe {
            let ptr = *(self.0.wrapping_add(76) as *const *const u8);
            (!ptr.is_null()).then_some(T::new_from_ptr(ptr))
        }
    }

    pub fn get_visitor_avatar<T: LogicAvatar>(&self) -> Option<T> {
        unsafe {
            let ptr = *(self.0.wrapping_add(80) as *const *const u8);
            (!ptr.is_null()).then_some(T::new_from_ptr(ptr))
        }
    }

    pub fn save_to_json(&self, json: &mut LogicJSONNode) {
        import!(logic_level_save_to_json(ptr: *const u8, json: *const u8) -> () = 0x1D8C56);
        logic_level_save_to_json(self.0, json.0);
    }
}
