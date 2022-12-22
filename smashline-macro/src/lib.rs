#![feature(asm)]
#![feature(const_loop)]
#![feature(const_if_match)]
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, token, AttrStyle, Attribute, Ident};

mod acmd;
mod acmd_lang;
mod attrs;
mod callbacks;
mod derive;
mod hook;
mod status;

use attrs::*;
use hook::*;

pub(crate) fn remove_mut(arg: &syn::FnArg) -> syn::FnArg {
    let mut arg = arg.clone();
    if let syn::FnArg::Typed(ref mut arg) = arg {
        if let syn::Pat::Ident(ref mut arg) = *arg.pat {
            arg.by_ref = None;
            arg.mutability = None;
            arg.subpat = None;
        }
    }
    arg
}

pub(crate) fn get_ident(arg: &syn::FnArg) -> syn::Ident {
    if let syn::FnArg::Typed(arg) = arg {
        if let syn::Pat::Ident(arg) = &*arg.pat {
            return arg.ident.clone();
        }
    }
    panic!("Agent frames require arguments to be named.")
}

pub(crate) fn new_attr(attr_name: &str, args: Option<&str>) -> syn::Attribute {
    let tokens = if let Some(args) = args {
        args.parse().unwrap()
    } else {
        TokenStream2::new()
    };
    syn::Attribute {
        pound_token: token::Pound {
            spans: [Span::call_site()],
        },
        style: AttrStyle::Outer,
        bracket_token: token::Bracket {
            span: Span::call_site(),
        },
        path: Ident::new(attr_name, Span::call_site()).into(),
        tokens,
    }
}

#[proc_macro_attribute]
pub fn hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    hook::hook(attrs, input)
}

#[proc_macro]
pub fn install_hook(input: TokenStream) -> TokenStream {
    hook::install_hook(input)
}

#[proc_macro_derive(LuaStruct)]
pub fn derive_lua_struct(item: TokenStream) -> TokenStream {
    derive::derive_lua_struct(item)
}

#[proc_macro_attribute]
pub fn acmd_script(attrs: TokenStream, input: TokenStream) -> TokenStream {
    acmd::acmd_script(attrs, input)
}

#[proc_macro]
pub fn install_acmd_script(input: TokenStream) -> TokenStream {
    acmd::install_acmd_script(input)
}

#[proc_macro_attribute]
pub fn status_script(attrs: TokenStream, input: TokenStream) -> TokenStream {
    status::status_script(attrs, input)
}

#[proc_macro_attribute]
pub fn common_status_script(attrs: TokenStream, input: TokenStream) -> TokenStream {
    status::common_status_script(attrs, input)
}

#[proc_macro]
pub fn install_status_script(input: TokenStream) -> TokenStream {
    status::install_status_script(input)
}

#[proc_macro_attribute]
pub fn fighter_frame(attrs: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_frame(attrs, input, true)
}

#[proc_macro_attribute]
pub fn weapon_frame(attrs: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_frame(attrs, input, false)
}

#[proc_macro_attribute]
pub fn agent_frame_main(attrs: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_frame_main(attrs, input)
}

#[proc_macro]
pub fn install_agent_frame(input: TokenStream) -> TokenStream {
    callbacks::install_agent_frame(input)
}

#[proc_macro_attribute]
pub fn fighter_reset(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_reset(input, true)
}

#[proc_macro_attribute]
pub fn agent_reset(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_reset(input, false)
}

#[proc_macro]
pub fn install_agent_reset(input: TokenStream) -> TokenStream {
    callbacks::install_agent_reset(input)
}

#[proc_macro_attribute]
pub fn installer(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut usr_fn = parse_macro_input!(input as syn::ItemFn);
    usr_fn.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern {
            span: Span::call_site(),
        },
        name: Some(syn::LitStr::new("Rust", Span::call_site())),
    });
    quote! (
        #[export_name = "smashline_install"]
        #usr_fn
    )
    .into()
}

#[proc_macro_attribute]
pub fn uninstaller(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut usr_fn = parse_macro_input!(input as syn::ItemFn);
    usr_fn.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern {
            span: Span::call_site(),
        },
        name: Some(syn::LitStr::new("Rust", Span::call_site())),
    });
    quote! (
        #[export_name = "smashline_uninstall"]
        #usr_fn
    )
    .into()
}

#[proc_macro_attribute]
pub fn fighter_frame_callback(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_frame_callback(input, true)
}

#[proc_macro_attribute]
pub fn weapon_frame_callback(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_frame_callback(input, false)
}

#[proc_macro_attribute]
pub fn agent_frame_main_callback(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_frame_main_callback(input)
}

#[proc_macro]
pub fn install_agent_frame_callback(input: TokenStream) -> TokenStream {
    callbacks::install_agent_frame_callback(input)
}

#[proc_macro]
pub fn generate_acmd_is_execute(input: TokenStream) -> TokenStream {
    acmd_lang::generate_acmd_is_execute(input)
}

#[proc_macro]
pub fn acmd(input: TokenStream) -> TokenStream {
    acmd_lang::acmd(input)
}

#[proc_macro_attribute]
pub fn fighter_init(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_init_callback(input, true)
}

#[proc_macro_attribute]
pub fn agent_init(_: TokenStream, input: TokenStream) -> TokenStream {
    callbacks::agent_init_callback(input, false)
}

#[proc_macro_attribute]
pub fn raw_inline_hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    hook::raw_inline_hook(attrs, input)
}

#[proc_macro_attribute]
pub fn raw_hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    hook::raw_hook(attrs, input)
}

#[proc_macro]
pub fn development_state_tracker(input: TokenStream) -> TokenStream {
    hook::development_state_tracker(input)
}

#[proc_macro]
pub fn development_state_installed(_: TokenStream) -> TokenStream {
    quote! {
        unsafe {
            crate::set_dev_plugin_available(true)
        }
    }
    .into()
}

#[proc_macro]
pub fn development_state_uninstalled(_: TokenStream) -> TokenStream {
    quote! {
        unsafe {
            crate::set_dev_plugin_available(false)
        }
    }
    .into()
}

#[proc_macro]
pub fn install_agent_init_callback(input: TokenStream) -> TokenStream {
    callbacks::install_agent_init_callback(input)
}

#[proc_macro]
pub fn install_raw_hook(input: TokenStream) -> TokenStream {
    hook::install_raw_hook(input)
}
