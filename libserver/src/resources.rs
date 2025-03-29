use std::ffi::CString;

use crate::{import, logic::json::LogicJSONNode, malloc, sc_string::ScString};

pub struct ResourceManager;

impl ResourceManager {
    pub fn get_json(path: &str) -> LogicJSONNode {
        import!(resource_manager_get_json(path: *const i8) -> *const u8 = 0x246802);
        LogicJSONNode(resource_manager_get_json(
            CString::new(path).unwrap().as_ptr(),
        ))
    }
}

#[repr(transparent)]
struct DataLoaderFactory(*const u8);

impl DataLoaderFactory {
    pub fn new() -> Self {
        import!(data_loader_factory_ctor(ptr: *const u8) -> () = 0x244C8E);

        let instance = malloc(4);
        data_loader_factory_ctor(instance);
        Self(instance)
    }
}

#[repr(transparent)]
struct ResourceListener(*const u8);

impl ResourceListener {
    pub fn new() -> Self {
        import!(resource_listener_ctor(ptr: *const u8) -> () = 0x244968);

        let instance = malloc(20);
        resource_listener_ctor(instance);
        Self(instance)
    }

    pub fn add_file(&self, name: &str) {
        import!(resource_listener_add_file(ptr: *const u8, name: *const u8, a3: i32, a4: i32, a5: i32, a6: i32) -> () = 0x2477FC);
        resource_listener_add_file(self.0, ScString::from(name).0, -1, -1, -1, -1);
    }

    pub fn start_loading(&self) {
        import!(resource_listener_start_loading(ptr: *const u8) -> () = 0x247D98);
        resource_listener_start_loading(self.0);
    }
}

pub fn init() {
    const CSV_FILES: &[&str] = &[
        "logic/buildings.csv",
        "logic/locales.csv",
        "logic/resources.csv",
        "logic/characters.csv",
        "csv/animations.csv",
        "logic/projectiles.csv",
        "csv/texts.csv",
        "csv/texts_patch.csv",
        "logic/regions.csv",
        "logic/building_classes.csv",
        "logic/obstacles.csv",
        "logic/effects.csv",
        "csv/particle_emitters.csv",
        "logic/experience_levels.csv",
        "logic/traps.csv",
        "logic/alliance_badges.csv",
        "logic/alliance_badge_layers.csv",
        "logic/globals.csv",
        "csv/client_globals.csv",
        "logic/townhall_levels.csv",
        "logic/alliance_portal.csv",
        "logic/npcs.csv",
        "logic/decos.csv",
        "csv/resource_packs.csv",
        "logic/shields.csv",
        "logic/missions.csv",
        "csv/billing_packages.csv",
        "logic/achievements.csv",
        "csv/credits.csv",
        "csv/faq.csv",
        "logic/spells.csv",
        "csv/hints.csv",
        "logic/heroes.csv",
        "logic/leagues.csv",
        "csv/news.csv",
        "logic/war.csv",
        "logic/alliance_levels.csv",
        "csv/helpshift.csv",
    ];

    const NPCS_COUNT: usize = 48;
    const PREBASES_COUNT: usize = 11;

    import!(resource_manager_init(factory: DataLoaderFactory, a2: *const u8) -> () = 0x2483E2);
    import!(resource_manager_resource_to_load() -> i32 = 0x2449E2);
    import!(resource_manager_load_next_resource() -> () = 0x246BDC);
    import!(logic_data_tables_init() -> () = 0x1AC81E);
    import!(logic_resources_create_data_table_resources_array() -> *const u8 = 0x1BC2A4);
    import!(resource_manager_get_csv(csv: *const u8) -> *const u8 = 0x24690A);
    import!(logic_resources_load(data_table_resources_array: *const u8, index: i32, csv: *const u8) -> *const u8 = 0x1BC154);

    let data_loader_factory = DataLoaderFactory::new();
    resource_manager_init(data_loader_factory, [0x00].as_ptr());
    logic_data_tables_init();

    let listener = ResourceListener::new();
    listener.add_file("level/starting_home.json");

    for i in 1..=NPCS_COUNT {
        listener.add_file(&format!("level/npc{i}.json"));
    }

    for i in 1..=PREBASES_COUNT {
        listener.add_file(&format!("level/townhall{i}.json"));
    }

    listener.add_file("level/tutorial_npc.json");
    listener.add_file("level/tutorial_npc2.json");

    for path in CSV_FILES {
        listener.add_file(path);
    }

    listener.start_loading();

    while resource_manager_resource_to_load() != 0 {
        resource_manager_load_next_resource();
    }

    let data_table_resources_array = logic_resources_create_data_table_resources_array();
    for (index, path) in CSV_FILES.iter().enumerate() {
        let csv = resource_manager_get_csv(ScString::from(path).0);
        logic_resources_load(data_table_resources_array, index as i32, csv);
    }
}
