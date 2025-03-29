use crate::{import, malloc, sc_string::ScString};

#[repr(transparent)]
pub struct LogicCommand(pub *const u8);

impl LogicCommand {
    pub fn get_command_type(&self) -> i32 {
        let get_command_type = unsafe {
            std::mem::transmute::<_, extern "C" fn(*const u8) -> i32>(
                *((*(self.0 as *const usize) + 16) as *const usize),
            )
        };

        get_command_type(self.0)
    }

    pub fn get_execute_sub_tick(&self) -> i32 {
        unsafe { *(self.0.wrapping_add(4) as *const i32) }
    }
}

#[repr(transparent)]
pub struct LogicCommandManager(pub *const u8);

impl LogicCommandManager {
    pub fn add_command(&self, command: &LogicCommand) {
        import!(logic_command_manager_add_command(ptr: *const u8, command: *const u8) -> () = 0x191888);
        logic_command_manager_add_command(self.0, command.0);
    }
}

pub struct LogicChangeAvatarNameCommand(pub LogicCommand);

impl LogicChangeAvatarNameCommand {
    pub fn new() -> Self {
        import!(logic_change_avatar_name_command_ctor(ptr: *const u8) -> () = 0x1E5762);
        let instance = malloc(20);
        logic_change_avatar_name_command_ctor(instance);
        Self(LogicCommand(instance))
    }

    pub fn set_avatar_name(&mut self, name: &str) {
        unsafe {
            *(self.0 .0.wrapping_add(12) as *mut usize) = ScString::from(name).0 as usize;
        }
    }

    pub fn set_name_change_state(&mut self, value: i32) {
        unsafe {
            *(self.0 .0.wrapping_add(16) as *mut i32) = value;
        }
    }
}
