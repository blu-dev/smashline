use syn::{parse_quote, parse_macro_input, Token, token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use crate::attrs::*;
use crate::new_attr;

fn generate_install_fn(usr_fn_name: &syn::Ident, orig: &syn::Ident, bind_fn_name: &syn::Ident, attrs: &AcmdAttrs) -> syn::ItemFn {
    let install_name = quote::format_ident!("{}_smashline_acmd_script_install", usr_fn_name);
    
    let agent = &attrs.agent;

    let mut install_fn: syn::ItemFn = parse_quote! {
        #[allow(non_snake_case)]
        #[allow(unused_unsafe)]
        pub fn #install_name() {
            let agent = unsafe { #agent };
        }
    };

    let category = &attrs.category;
    let low_priority = &attrs.low_priority;

    let can_call_orig = attrs.scripts.len() == 1;

    for script in attrs.scripts.iter() {
        if can_call_orig {
            install_fn.block.stmts.push(parse_quote! {
                unsafe { smashline::replace_acmd_script(agent, #script, Some(&mut #orig), #category, #low_priority, #bind_fn_name as *const extern "C" fn()) };
            })
        } else {
            install_fn.block.stmts.push(parse_quote! {
                unsafe { smashline::replace_acmd_script(agent, #script, None, #category, #low_priority, #bind_fn_name as *const extern "C" fn()) };
            })
        }
    }

    install_fn
}

fn generate_original_macro(usr_fn_name: &syn::Ident, orig_name: &syn::Ident, valid: bool) -> syn::Stmt {
    if valid {
        parse_quote! {
            macro_rules! original {
                ($agent:ident) => {
                    {
                        #[allow(unused_unsafe)]
                        if true {
                            unsafe {
                                if #orig_name.is_null() {
                                    panic!("Error calling ACMD script {}, original function not in memory.", stringify!(#usr_fn_name));
                                }
                                std::mem::transmute::<_, extern "C" fn(&mut smash::lua2cpp::L2CAgentBase, *mut smash::lib::utility::Variadic)>(#orig_name)($agent, &0u64 as *const u64 as _);
                            }
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
        }
    } else {
        parse_quote! {
            macro_rules! original {
                ($agent:ident) => {
                    {
                        compile_error!("ACMD replacements which replace multiple scripts cannot call `original!`")
                    }
                }
            }
        }
    }
}

pub fn install_acmd_script(input: TokenStream) -> TokenStream {
    let usr_fn_name = parse_macro_input!(input as syn::Ident);
    let install_name = quote::format_ident!("{}_smashline_acmd_script_install", usr_fn_name);
    quote!(
        unsafe { #install_name(); }
    ).into()
}

pub fn acmd_script(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as AcmdAttrs);
    let mut usr_fn = parse_macro_input!(input as syn::ItemFn);

    let usr_fn_name = usr_fn.sig.ident.clone();

    let usr_new_name = quote::format_ident!("{}_smashline_acmd_script_usr", usr_fn_name);
    let bind_fn_name = quote::format_ident!("{}_smashline_acmd_script_bind", usr_fn_name);
    let orig_name = quote::format_ident!("{}_smashline_acmd_script_orig", usr_fn_name);

    usr_fn.sig.ident = usr_new_name.clone();;
    usr_fn.attrs.push(
        new_attr("inline", Some("(always)"))
    );

    let orig_macro = generate_original_macro(&usr_fn_name, &orig_name, attrs.scripts.len() == 1);

    usr_fn.block.stmts.insert(0, orig_macro);

    let install_fn = generate_install_fn(&usr_fn_name, &orig_name, &bind_fn_name, &attrs);

    quote!(
        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        static mut #orig_name: *const extern "C" fn() = 0 as _;

        #usr_fn

        #install_fn

        #[allow(non_snake_case)]
        #[inline(never)]
        unsafe extern "C" fn #bind_fn_name(agent: &mut smash::lua2cpp::L2CAgentBase, _: &mut smash::lib::utility::Variadic) {
            #usr_new_name(agent);
            std::arch::asm!(r#"
            b #0x8
            .byte 0xE5, 0xB1, 0x00, 0xB0
            "#)
        }
    ).into()
}
