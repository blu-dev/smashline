#![feature(proc_macro_hygiene)]

use smash::phx::Hash40;
use smash::lib::{L2CValue, LuaConst};
use smash::lua2cpp::*;

pub use smashline_macro::*;

type FighterFrame = extern "C" fn(&mut L2CFighterCommon) -> L2CValue;
type WeaponFrame = extern "C" fn(&mut L2CFighterBase) -> L2CValue;

#[macro_export]
macro_rules! install_hooks {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_hook!($fn);
        )*
    }
}

#[macro_export]
macro_rules! install_acmd_scripts {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_acmd_script!($fn);
        )*
    }
}

#[macro_export]
macro_rules! install_status_scripts {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_status_script!($fn);
        )*
    }
}

#[macro_export]
macro_rules! install_agent_frames {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_agent_frame!($fn);
        )*
    }
}

pub enum StaticSymbol {
    Resolved(usize),
    Unresolved(&'static str)
}

pub enum LuaConstant {
    Symbolic(LuaConst),
    Evaluated(i32)
}

#[allow(non_camel_case_types)]
pub enum AcmdCategory {
    ACMD_GAME,
    ACMD_EFFECT,
    ACMD_SOUND,
    ACMD_EXPRESSION
}

pub use AcmdCategory::*;

extern "Rust" {
    pub fn replace_symbol(module: &str, symbol: &str, replace: *const extern "C" fn(), original: Option<&'static mut *const extern "C" fn()>);
    pub fn replace_static_symbol(symbol: StaticSymbol, replace: *const extern "C" fn(), original: Option<&'static mut *const extern "C" fn()>);

    pub fn replace_acmd_script(agent: Hash40, script: Hash40, original: Option<&'static mut *const extern "C" fn()>, category: AcmdCategory, low_priority: bool, bind_fn: *const extern "C" fn());
    pub fn replace_status_script(agent: Hash40, script: LuaConstant, condition: LuaConstant, original: Option<&'static mut *const extern "C" fn()>, low_priority: bool, replacement: *const extern "C" fn());

    pub fn replace_fighter_frame(agent: LuaConstant, original: Option<&'static mut *const extern "C" fn()>, replacement: FighterFrame);
    pub fn replace_weapon_frame(agent: LuaConstant, original: Option<&'static mut *const extern "C" fn()>, replacement: WeaponFrame);
}