use std::{
    ffi::c_void,
    net::{SocketAddr, TcpListener},
    os::fd::IntoRawFd,
    sync::{Arc, LazyLock, Mutex},
    thread,
};

use byte_stream::ByteStream;
use database::DatabaseConnection;
use ffi_util::import;

use logic::avatar::*;
use logic::home::LogicClientHome;
use logic::json::LogicJSONNode;
use logic::mode::LogicGameMode;

use math::LogicLong;
use network::PiranhaMessage;
use rand::RngCore;
use resources::ResourceManager;
use sc_string::StringBuilder;
use tracing::{error, info, warn};

mod array_list;
mod byte_stream;
mod database;
mod ffi_util;
mod jni_util;
mod logic;
mod math;
mod message;
mod network;
mod resources;
mod sc_string;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: jni::JavaVM, _: *mut c_void) -> jni::sys::jint {
    init_tracing();

    let env = vm.get_env().unwrap();
    let package_name = jni_util::get_package_name(env);

    info!("OnLoad()");
    info!("PackageName: {package_name}");

    thread::spawn(move || {
        server_main(ServerConfig {
            database_path: format!("/data/data/{package_name}/magic.db"),
        })
    });

    jni::sys::JNI_VERSION_1_6
}

struct ServerConfig {
    pub database_path: String,
}

pub fn malloc(amount: usize) -> *const u8 {
    unsafe  extern  "C" {
        fn malloc(amount: usize) -> *const u8;
    }

    unsafe {
        malloc(amount)
    }
} 

//import!(malloc(amount: usize) -> *const u8 = 0x56A20);
import!(free(ptr: *const u8) -> () = 0x54590);

fn server_main(config: ServerConfig) {
    const TCP_ADDR: &str = "0.0.0.0:9339";

    info!("starting server...");

    ffi_util::disable_event_tracker();
    resources::init();

    info!("successfully initialized resources");

    let db = DatabaseConnection::connect(&config.database_path).unwrap_or_else(|err| {
        error!("DatabaseConnection::connect failed: {err}");
        panic!();
    });

    let db = Arc::new(Mutex::new(db));

    let listener = TcpListener::bind(TCP_ADDR).unwrap();
    info!("server is listening at {TCP_ADDR}");

    while let Ok((stream, addr)) = listener.accept() {
        info!("new connection from {addr}");

        let fd = stream.into_raw_fd();
        let db = Arc::clone(&db);

        thread::spawn(move || receive_loop(fd, addr, db));
    }
}

fn receive_loop(fd: i32, addr: SocketAddr, db: Arc<Mutex<DatabaseConnection>>) {
    use network::{LogicMagicMessageFactory, Messaging, RC4Encrypter};

    static MESSAGE_FACTORY: LazyLock<LogicMagicMessageFactory> =
        LazyLock::new(|| LogicMagicMessageFactory::new());

    let mut messaging = Messaging::new(fd);
    messaging.set_message_factory(&*MESSAGE_FACTORY);
    messaging.set_encrypters(
        RC4Encrypter::new(LogicMagicMessageFactory::RC4_KEY, "nonce"),
        RC4Encrypter::new(LogicMagicMessageFactory::RC4_KEY, "nonce"),
    );

    let mut session = PlayerSession {
        messaging,
        account_id: LogicLong::new(0, 0),
        logic_game_mode: None,
        saved_home_json: None,
    };

    while session.messaging.get_connection().is_connected {
        session.messaging.on_receive();
        while let Some(message) = session.messaging.next_message() {
            handle_message(&mut session, db.as_ref(), message);
        }
    }

    info!("client from {addr} disconnected");
}

struct PlayerSession {
    pub messaging: network::Messaging,
    pub account_id: LogicLong,
    pub logic_game_mode: Option<LogicGameMode>,
    pub saved_home_json: Option<String>,
}

fn handle_message(
    session: &mut PlayerSession,
    db: &Mutex<DatabaseConnection>,
    message: PiranhaMessage,
) {
    match message.get_message_type() {
        10101 => handle_login_message(session, db, message),
        10108 => handle_keep_alive_message(session, message),
        10212 => handle_change_avatar_name_message(session, message),
        14101 => handle_go_home_message(session, message),
        14102 => handle_end_client_turn_message(session, db, message),
        14134 => handle_attack_npc_message(session, message),
        unhandled => warn!("unhandled message: {unhandled}"),
    }

    session.messaging.on_wakeup();
}

fn handle_login_message(
    session: &mut PlayerSession,
    db: &Mutex<DatabaseConnection>,
    message: PiranhaMessage,
) {
    use message::{ExtendedSetEncryptionMessage, LoginMessage, LoginOkMessage, OwnHomeDataMessage};
    use network::{LogicMagicMessageFactory, RC4Encrypter};

    let login_message = LoginMessage(message);

    info!(
        "LoginMessage received, account_id: {}, pass_token: {:?}",
        login_message.get_account_id(),
        login_message.get_pass_token()
    );

    let Ok(Some(player_data)) = db
        .lock()
        .unwrap()
        .fetch_or_create_player(&login_message.get_account_id())
    else {
        warn!(
            "Login: player with id {} was not found in the database",
            login_message.get_account_id()
        );
        return;
    };

    if !login_message.get_account_id().is_zero() {
        let Some(pass_token) = login_message.get_pass_token() else {
            error!(
                "Login: received null pass token with non-zero account id: {}",
                login_message.get_account_id()
            );
            return;
        };

        if pass_token.to_string() != player_data.pass_token {
            warn!(
                "Login: pass token mismatch, account id: {}",
                login_message.get_account_id()
            );
            return;
        }
    }

    let mut set_encryption_message = ExtendedSetEncryptionMessage::new();
    let mut nonce = [0u8; 64];
    rand::rng().fill_bytes(&mut nonce);
    set_encryption_message.set_nonce(&nonce);
    set_encryption_message.set_scrambler_method(1);
    session.messaging.send(set_encryption_message.0);
    session.messaging.on_wakeup();

    network::Messaging::scramble_nonce_using_mersenne_twister(
        login_message.get_scrambler_seed(),
        &mut nonce,
    );

    let encrypter = RC4Encrypter::new_with_nonce_bytes(LogicMagicMessageFactory::RC4_KEY, &nonce);
    let decrypter = RC4Encrypter::new_with_nonce_bytes(LogicMagicMessageFactory::RC4_KEY, &nonce);
    session.messaging.set_encrypters(encrypter, decrypter);

    let mut login_ok_message = LoginOkMessage::new();
    login_ok_message.set_account_id(player_data.id.clone());
    login_ok_message.set_home_id(player_data.id.clone());
    login_ok_message.set_pass_token(&player_data.pass_token);
    login_ok_message.set_server_major_version(8);
    login_ok_message.set_server_build(67);
    login_ok_message.set_content_version(0);
    login_ok_message.set_server_environment("dev");

    let mut own_home_data_message = OwnHomeDataMessage::new();

    let mut logic_client_home = LogicClientHome::new();
    logic_client_home.set_home_json(&player_data.home_json);

    let mut logic_client_avatar = LogicClientAvatar::new();

    let data = rbase64::decode(&player_data.client_avatar_blob).unwrap();
    let mut byte_stream = ByteStream::from(&data);
    logic_client_avatar.decode(&mut byte_stream);

    let mut logic_game_mode = LogicGameMode::new();
    logic_game_mode.load_home_state(&logic_client_home, &logic_client_avatar, 0);

    own_home_data_message.set_logic_client_home({
        let mut logic_client_home = LogicClientHome::new();
        logic_client_home.set_home_json(&player_data.home_json);
        logic_client_home
    });

    own_home_data_message.set_logic_client_avatar(logic_game_mode.get_cloned_home_owner().unwrap());

    session.account_id = player_data.id;
    session.logic_game_mode = Some(logic_game_mode);
    session.saved_home_json = Some(player_data.home_json);

    session.messaging.send(login_ok_message.0);
    session.messaging.send(own_home_data_message.0);

    info!("successfully logged in");
}

fn handle_keep_alive_message(session: &mut PlayerSession, _message: PiranhaMessage) {
    session
        .messaging
        .send(message::KeepAliveServerMessage::new().0);
}

fn handle_go_home_message(session: &mut PlayerSession, _message: PiranhaMessage) {
    use message::OwnHomeDataMessage;

    let Some(logic_game_mode) = session.logic_game_mode.as_mut() else {
        error!("received GoHomeMessage while LogicGameMode is NULL!");
        return;
    };

    if logic_game_mode.get_state() == 1 {
        error!("received GoHomeMessage being already in home state!");
        return;
    }

    let Some(home_json) = session.saved_home_json.as_ref() else {
        error!("received GoHomeMessage while saved_home_json is NULL!");
        return;
    };

    let mut own_home_data_message = OwnHomeDataMessage::new();

    let mut logic_client_home = LogicClientHome::new();
    logic_client_home.set_home_json(home_json);

    let logic_client_avatar = logic_game_mode.get_cloned_visitor().unwrap();

    let mut logic_game_mode = LogicGameMode::new();
    logic_game_mode.load_home_state(&logic_client_home, &logic_client_avatar, 0);

    own_home_data_message.set_logic_client_home({
        let mut logic_client_home = LogicClientHome::new();
        logic_client_home.set_home_json(home_json);
        logic_client_home
    });

    own_home_data_message.set_logic_client_avatar(logic_game_mode.get_cloned_home_owner().unwrap());

    session.logic_game_mode = Some(logic_game_mode);
    session.messaging.send(own_home_data_message.0);
}

fn handle_end_client_turn_message(
    session: &mut PlayerSession,
    db: &Mutex<DatabaseConnection>,
    message: PiranhaMessage,
) {
    use message::{EndClientTurnMessage, OutOfSyncMessage};

    let message = EndClientTurnMessage(message);

    let Some(logic_game_mode) = session.logic_game_mode.as_mut() else {
        error!("received EndClientTurnMessage while LogicGameMode is NULL!");
        return;
    };

    info!(
        "EndClientTurnMessage received: sub_tick: {}, checksum: {}",
        message.get_sub_tick(),
        message.get_checksum()
    );

    let client_sub_tick = message.get_sub_tick();
    while logic_game_mode.get_level().get_time().sub_tick < client_sub_tick {
        if let Some(commands) = message.get_commands() {
            let cur_sub_tick = logic_game_mode.get_level().get_time().sub_tick;
            for command in commands.as_slice().iter() {
                if command.get_execute_sub_tick() == cur_sub_tick {
                    info!(
                        "received command: {}, exec sub tick: {}",
                        command.get_command_type(),
                        command.get_execute_sub_tick()
                    );

                    logic_game_mode.get_command_manager().add_command(command);
                }
            }
        }

        logic_game_mode.update_one_sub_tick();
    }

    let mut debug_json = LogicJSONNode::new_json_object();
    let checksum = logic_game_mode.calculate_checksum(Some(&mut debug_json), false);
    if checksum != message.get_checksum() {
        error!("Client and server are out of sync! sub_tick: {}, server checksum: {}, client checksum: {}", message.get_sub_tick(), checksum, message.get_checksum());

        let mut sb = StringBuilder::new();
        debug_json.write_to_string(&mut sb);
        info!("{}", sb.to_string());

        let mut out_of_sync_message = OutOfSyncMessage::new();
        out_of_sync_message.set_server_checksum(checksum);
        out_of_sync_message.set_client_checksum(message.get_checksum());
        out_of_sync_message.set_sub_tick(message.get_sub_tick());
        session.messaging.send(out_of_sync_message.0);
    }

    if logic_game_mode.get_state() == 1 {
        let mut string_builder = StringBuilder::new();
        let mut home_json_object = LogicJSONNode::new_json_object();

        logic_game_mode.save_to_json(&mut home_json_object);
        home_json_object.write_to_string(&mut string_builder);

        let home_json = string_builder.to_string();

        if let Err(err) = db.lock().unwrap().save_player_data(
            &session.account_id,
            &home_json,
            &logic_game_mode.get_cloned_home_owner().unwrap(),
        ) {
            error!("failed to save player data: {err}");
        }

        session.saved_home_json = Some(home_json);
    }
}

fn handle_attack_npc_message(session: &mut PlayerSession, message: PiranhaMessage) {
    use message::{AttackNpcMessage, NpcDataMessage};

    let message = AttackNpcMessage(message);

    let Some(logic_game_mode) = session.logic_game_mode.as_mut() else {
        error!("received AttackNpcMessage while LogicGameMode is NULL!");
        return;
    };

    let Some(home_owner_avatar) = logic_game_mode.get_level().get_home_owner_avatar() else {
        error!("received AttackNpcMessage while home_owner_avatar is NULL!");
        return;
    };

    let mut string_builder = StringBuilder::new();
    ResourceManager::get_json(
        &message
            .get_npc_data()
            .get_level_json_file_name()
            .to_string(),
    )
    .write_to_string(&mut string_builder);

    let mut logic_client_home = LogicClientHome::new();
    logic_client_home.set_home_json(&string_builder.to_string());

    let mut logic_npc_avatar = LogicNpcAvatar::new();
    logic_npc_avatar.set_npc_data(&message.get_npc_data());

    let mut logic_game_mode = LogicGameMode::new();
    logic_game_mode.load_npc_attack_state(
        &logic_client_home,
        &logic_npc_avatar,
        &home_owner_avatar,
        0,
    );

    let mut npc_data_message = NpcDataMessage::new();
    npc_data_message.set_level_json(&string_builder.to_string());
    npc_data_message.set_logic_npc_avatar(&logic_game_mode.get_cloned_home_owner().unwrap());
    npc_data_message.set_logic_client_avatar(&logic_game_mode.get_cloned_visitor().unwrap());

    session.logic_game_mode = Some(logic_game_mode);
    session.messaging.send(npc_data_message.0);
}

fn handle_change_avatar_name_message(session: &mut PlayerSession, message: PiranhaMessage) {
    use logic::command::LogicChangeAvatarNameCommand;
    use message::{AvailableServerCommandMessage, ChangeAvatarNameMessage};

    let message = ChangeAvatarNameMessage(message);

    let mut logic_change_avatar_name_command = LogicChangeAvatarNameCommand::new();
    logic_change_avatar_name_command.set_avatar_name(&message.get_avatar_name());
    logic_change_avatar_name_command.set_name_change_state(1);

    let mut available_server_command_message = AvailableServerCommandMessage::new();
    available_server_command_message.set_server_command(&logic_change_avatar_name_command.0);

    session.messaging.send(available_server_command_message.0);
}

fn init_tracing() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_android::layer("MAGIC-SERVER").unwrap())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}
