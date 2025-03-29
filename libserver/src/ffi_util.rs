use std::{ffi::c_void, sync::LazyLock};
use std::fs::File;
use std::io::{self, BufRead};

pub static LIBG_BASE: LazyLock<usize> = LazyLock::new(|| get_module_base("libg.so").unwrap());

macro_rules! import {
    ($name:ident($($arg_name:ident: $arg_type:ty),*) -> $ret_type:ty = $rva:expr) => {
        pub fn $name($($arg_name: $arg_type,)*) -> $ret_type {
            unsafe {
                type FuncType = unsafe extern "cdecl" fn($($arg_type,)*) -> $ret_type;
                 ::std::mem::transmute::<usize, FuncType>(*crate::ffi_util::LIBG_BASE + $rva)($($arg_name,)*)
            }
        }
    };
}

pub fn disable_event_tracker() {
    // Causes crashes in logic functions due to being not initialized
    // useless SC analytics.

    const EVENT_TRACKER_FUNCTIONS: &[i32] = &[0x1DF9F4, 0x1DF664, 0x1DF7E8, 0x1DF6FC, 0x26CC51];
    const TRACK_FUNCTIONS: &[i32] = &[0x26D41B, 0x1DFC4C, 0x1DF966];

    unsafe {
        for &addr in EVENT_TRACKER_FUNCTIONS.iter().chain(TRACK_FUNCTIONS) {
            let page_size = libc::sysconf(libc::_SC_PAGE_SIZE);
            let addr = LIBG_BASE.wrapping_add(addr as usize);
            libc::mprotect(
                ((addr as i32) & !(page_size - 1)) as *mut c_void,
                page_size as usize,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            );

            std::slice::from_raw_parts_mut(addr as *mut u8, 1).copy_from_slice(&[0xC3]);
        }
    }
}

pub(crate) use import;

pub fn get_module_base(shared_object_name: &str) -> Option<usize> {
    let path = "/proc/self/maps";
    let file = File::open(path).ok()?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.ok()?;
        if line.contains(shared_object_name) {
            let address_str = line.split_whitespace().next().unwrap_or("");
            let address = usize::from_str_radix(&address_str.split('-').next().unwrap_or(""), 16)
                .ok()?;
            return Some(address);
        }
    }

    None
}