use crate::{
    array_list::LogicArrayList,
    import,
    logic::{
        avatar::{LogicClientAvatar, LogicNpcAvatar},
        command::LogicCommand,
        data::LogicNpcData,
        home::LogicClientHome,
    },
    malloc,
    network::PiranhaMessage,
    sc_string::ScString,
};

pub struct OwnHomeDataMessage(pub PiranhaMessage);

impl OwnHomeDataMessage {
    pub fn new() -> Self {
        import!(own_home_data_message_ctor(ptr: *const u8) -> () = 0x21790E);

        let instance = malloc(104);
        own_home_data_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }

    pub fn set_logic_client_home(&mut self, logic_client_home: LogicClientHome) {
        unsafe {
            *(self.0 .0.wrapping_add(68) as *mut usize) = logic_client_home.0 as usize;
        }
    }

    pub fn set_logic_client_avatar(&mut self, logic_client_avatar: LogicClientAvatar) {
        unsafe {
            *(self.0 .0.wrapping_add(72) as *mut usize) = logic_client_avatar.0 as usize;
        }
    }
}

pub struct EndClientTurnMessage(pub PiranhaMessage);

impl EndClientTurnMessage {
    pub fn get_sub_tick(&self) -> i32 {
        unsafe { *(self.0 .0.wrapping_add(52) as *const i32) }
    }

    pub fn get_checksum(&self) -> i32 {
        unsafe { *(self.0 .0.wrapping_add(56) as *const i32) }
    }

    pub fn get_commands(&self) -> Option<&LogicArrayList<LogicCommand>> {
        unsafe {
            let list_ptr =
                *(self.0 .0.wrapping_add(48) as *const *const LogicArrayList<LogicCommand>);
            (!list_ptr.is_null()).then_some(&*list_ptr)
        }
    }
}

pub struct OutOfSyncMessage(pub PiranhaMessage);

impl OutOfSyncMessage {
    pub fn new() -> Self {
        import!(out_of_sync_message_ctor(ptr: *const u8) -> () = 0x2175CE);
        let instance = malloc(64);
        out_of_sync_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }

    pub fn set_server_checksum(&mut self, value: i32) {
        unsafe { *(self.0 .0.wrapping_add(48) as *mut i32) = value }
    }

    pub fn set_client_checksum(&mut self, value: i32) {
        unsafe { *(self.0 .0.wrapping_add(52) as *mut i32) = value }
    }

    pub fn set_sub_tick(&mut self, value: i32) {
        unsafe { *(self.0 .0.wrapping_add(56) as *mut i32) = value }
    }
}

pub struct AvailableServerCommandMessage(pub PiranhaMessage);

impl AvailableServerCommandMessage {
    pub fn new() -> Self {
        import!(available_server_command_message_ctor(ptr: *const u8) -> () = 0x215520);
        let instance = malloc(100);
        available_server_command_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }

    pub fn set_server_command(&mut self, command: &LogicCommand) {
        unsafe { *(self.0 .0.wrapping_add(48) as *mut usize) = command.0 as usize }
    }
}

pub struct AttackNpcMessage(pub PiranhaMessage);

impl AttackNpcMessage {
    pub fn get_npc_data(&self) -> LogicNpcData {
        unsafe { LogicNpcData(*(self.0 .0.wrapping_add(48) as *const *const u8)) }
    }
}

pub struct NpcDataMessage(pub PiranhaMessage);

impl NpcDataMessage {
    pub fn new() -> Self {
        import!(npc_data_message_ctor(ptr: *const u8) -> () = 0x217340);
        let instance = malloc(64);
        npc_data_message_ctor(instance);
        Self(PiranhaMessage(instance))
    }

    pub fn set_level_json(&mut self, json: &str) {
        unsafe {
            *(self.0 .0.wrapping_add(52) as *mut usize) = ScString::from(json).0 as usize;
        }
    }

    pub fn set_logic_client_avatar(&mut self, value: &LogicClientAvatar) {
        unsafe {
            *(self.0 .0.wrapping_add(56) as *mut usize) = value.0 as usize;
        }
    }

    pub fn set_logic_npc_avatar(&mut self, value: &LogicNpcAvatar) {
        unsafe {
            *(self.0 .0.wrapping_add(60) as *mut usize) = value.0 as usize;
        }
    }
}
