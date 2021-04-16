#![feature(asm)]
#![feature(const_loop)]
#![feature(const_if_match)]
use syn::{Attribute, token, AttrStyle, Ident};
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Span};

mod acmd;
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
        pound_token: token::Pound { spans: [Span::call_site()]},
        style: AttrStyle::Outer,
        bracket_token: token::Bracket { span: Span::call_site() },
        path: Ident::new(attr_name, Span::call_site()).into(),
        tokens
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

#[proc_macro]
pub fn install_agent_frame(input: TokenStream) -> TokenStream {
    callbacks::install_agent_frame(input)
}