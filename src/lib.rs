#![feature(proc_macro_hygiene)]

use smash::phx::Hash40;
use smash::lib::{L2CValue, LuaConst};
use smash::lua2cpp::*;

pub use smashline_macro::*;

type FighterFrame = extern "C" fn(&mut L2CFighterCommon) -> L2CValue;
type AgentFrame = extern "C" fn(&mut L2CFighterBase) -> L2CValue;
type FighterFrameCallback = fn(&mut L2CFighterCommon);
type AgentFrameCallback = fn(&mut L2CFighterBase);
type FighterReset = fn(&mut L2CFighterCommon);
type AgentReset = fn(&mut L2CFighterBase);
type FighterInit = fn(&mut L2CFighterCommon);
type AgentInit = fn(&mut L2CFighterBase);

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

#[macro_export]
macro_rules! install_agent_resets {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_agent_reset!($fn);
        )*
    }
}

#[macro_export]
macro_rules! install_agent_frame_callbacks {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_agent_frame_callback!($fn); 
        )*
    }
}

#[macro_export]
macro_rules! install_agent_init_callbacks {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_agent_init_callback!($fn);
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
    pub fn replace_common_status_script(script: LuaConstant, condition: LuaConstant, original: Option<&'static mut *const extern "C" fn()>, replacement: *const extern "C" fn());

    pub fn replace_fighter_frame(agent: LuaConstant, original: Option<&'static mut *const extern "C" fn()>, replacement: FighterFrame);
    pub fn replace_weapon_frame(agent: LuaConstant, original: Option<&'static mut *const extern "C" fn()>, replacement: AgentFrame);
    pub fn replace_agent_frame_main(agent: LuaConstant, is_fighter: bool, original: Option<&'static mut *const extern "C" fn()>, replacement: AgentFrame);

    pub fn add_fighter_reset_callback(callback: FighterReset);
    pub fn add_agent_reset_callback(callback: AgentReset);

    pub fn add_fighter_frame_callback(callback: FighterFrameCallback);
    pub fn add_weapon_frame_callback(callback: AgentFrameCallback);
    pub fn add_agent_frame_main_callback(callback: AgentFrameCallback);

    pub fn add_fighter_init_callback(callback: FighterInit);
    pub fn add_agent_init_callback(callback: AgentInit);
}