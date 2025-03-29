use crate::{byte_stream::ByteStream, import, malloc};

use super::{
    avatar::{LogicAvatar, LogicClientAvatar, LogicNpcAvatar},
    command::LogicCommandManager,
    home::LogicClientHome,
    json::LogicJSONNode,
    level::LogicLevel,
};

#[repr(transparent)]
pub struct LogicGameMode(pub *const u8);

impl LogicGameMode {
    pub fn new() -> Self {
        import!(logic_game_mode_ctor(ptr: *const u8) -> () = 0x1DC75A);
        let instance = malloc(68);
        logic_game_mode_ctor(instance);

        Self(instance)
    }

    pub fn load_home_state(
        &mut self,
        logic_client_home: &LogicClientHome,
        logic_client_avatar: &LogicClientAvatar,
        seconds_since_last_save: i32,
    ) {
        import!(logic_game_mode_load_home_state(lgm: *const u8, lch: *const u8, lca: *const u8, ssls: i32, a5: i32, a6: i32, a7: i32) -> () = 0x1DE0C6);

        logic_game_mode_load_home_state(
            self.0,
            logic_client_home.0,
            logic_client_avatar.0,
            seconds_since_last_save,
            0,
            -1,
            -1,
        );
    }

    pub fn load_npc_attack_state(
        &mut self,
        logic_client_home: &LogicClientHome,
        logic_npc_avatar: &LogicNpcAvatar,
        logic_client_avatar: &LogicClientAvatar,
        seconds_since_last_save: i32,
    ) {
        import!(logic_game_mode_load_npc_attack_state(lgm: *const u8, lch: *const u8, lna: *const u8, lca: *const u8, ssls: i32) -> () = 0x1DCE08);

        logic_game_mode_load_npc_attack_state(
            self.0,
            logic_client_home.0,
            logic_npc_avatar.0,
            logic_client_avatar.0,
            seconds_since_last_save,
        );
    }

    pub fn get_cloned_home_owner<T: LogicAvatar>(&self) -> Option<T> {
        let avatar = self.get_level().get_home_owner_avatar::<T>()?;
        let mut stream = ByteStream::new(10);
        LogicAvatar::encode(&avatar, &mut stream);
        stream.reset_offset();
        let mut cloned_avatar = T::new();
        LogicAvatar::decode(&mut cloned_avatar, &mut stream);

        Some(cloned_avatar)
    }

    pub fn get_cloned_visitor<T: LogicAvatar>(&self) -> Option<T> {
        let avatar = self.get_level().get_visitor_avatar::<T>()?;
        let mut stream = ByteStream::new(10);
        LogicAvatar::encode(&avatar, &mut stream);
        stream.reset_offset();
        let mut cloned_avatar = T::new();
        LogicAvatar::decode(&mut cloned_avatar, &mut stream);

        Some(cloned_avatar)
    }

    pub fn update_one_sub_tick(&self) {
        import!(logic_game_mode_update_one_sub_tick(ptr: *const u8) -> () = 0x1DDA14);
        logic_game_mode_update_one_sub_tick(self.0);
    }

    pub fn calculate_checksum(
        &self,
        debug_json: Option<&mut LogicJSONNode>,
        include_game_objects: bool,
    ) -> i32 {
        import!(logic_game_mode_calculate_checksum(ptr: *const u8, debug_json: *const u8, include_game_objects: bool) -> i32 = 0x1DDE0C);

        logic_game_mode_calculate_checksum(
            self.0,
            debug_json.map(|ptr| ptr.0).unwrap_or(std::ptr::null()),
            include_game_objects,
        )
    }

    pub fn save_to_json(&self, json: &mut LogicJSONNode) {
        self.get_level().save_to_json(json);
    }

    pub fn get_command_manager(&self) -> LogicCommandManager {
        unsafe { LogicCommandManager(*(self.0.wrapping_add(20) as *const *const u8)) }
    }

    pub fn get_level(&self) -> LogicLevel {
        unsafe { LogicLevel(*(self.0.wrapping_add(16) as *const *const u8)) }
    }

    pub fn get_state(&self) -> i32 {
        unsafe { *(self.0 as *const i32) }
    }
}
