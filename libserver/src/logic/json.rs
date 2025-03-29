use crate::{import, malloc, sc_string::StringBuilder};

#[repr(transparent)]
pub struct LogicJSONNode(pub *const u8);

impl LogicJSONNode {
    pub fn new_json_object() -> Self {
        import!(logic_json_object_ctor(ptr: *const u8) -> () = 0x26A3A4);
        let instance = malloc(28);
        logic_json_object_ctor(instance);
        Self(instance)
    }

    pub fn write_to_string(&self, string_builder: &mut StringBuilder) {
        let write_to_string = unsafe {
            std::mem::transmute::<_, extern "C" fn(*const u8, *const u8)>(
                *((*(self.0 as *const usize) + 16) as *const usize),
            )
        };
        write_to_string(self.0, string_builder.0);
    }
}
