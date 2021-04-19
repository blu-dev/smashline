use syn::{parse_quote, parse_macro_input, Token, token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use crate::attrs::*;
use crate::{remove_mut, get_ident};

fn generate_fighter_install_fn(attrs: &AgentFrameAttrs, usr_fn_name: &syn::Ident, orig_name: &syn::Ident) -> TokenStream2 {
    let install_name = quote::format_ident!("{}_smashline_agent_frame_install", usr_fn_name);
    if let Some(agent) = &attrs.agent {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe { 
                    smashline::replace_fighter_frame(#agent, Some(&mut #orig_name), #usr_fn_name);
                }
            }
        ).into()
    } else {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::replace_symbol("common", "_ZN7lua2cpp16L2CFighterCommon31sys_line_system_control_fighterEv", #usr_fn_name as *const extern "C" fn(), Some(&mut #orig_name));
                }
            }
        ).into()
    }
}

fn generate_weapon_install_fn(attrs: &AgentFrameAttrs, usr_fn_name: &syn::Ident, orig_name: &syn::Ident) -> TokenStream2 {
    let install_name = quote::format_ident!("{}_smashline_agent_frame_install", usr_fn_name);
    if let Some(agent) = &attrs.agent {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::replace_weapon_frame(#agent, Some(&mut #orig_name), #usr_fn_name);
                }
            }
        ).into()
    } else {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::replace_symbol("common", "_ZN7lua2cpp14L2CFighterBase23sys_line_system_controlEv", #usr_fn_name as *const extern "C" fn(), Some(&mut #orig_name));
                }
            }
        ).into()
    }
}

pub fn install_agent_frame(input: TokenStream) -> TokenStream {
    let usr_fn_name = parse_macro_input!(input as syn::Ident);
    let install_name = quote::format_ident!("{}_smashline_agent_frame_install", usr_fn_name);
    quote!(
        unsafe { #install_name() };
    ).into()
}

pub fn agent_frame(attrs: TokenStream, input: TokenStream, is_fighter: bool) -> TokenStream {
    let attrs = parse_macro_input!(attrs as AgentFrameAttrs);
    let mut usr_fn = parse_macro_input!(input as syn::ItemFn);

    let usr_fn_name = usr_fn.sig.ident.clone();

    usr_fn.sig.abi = Some(syn::Abi {
        extern_token: token::Extern { span: Span::call_site() },
        name: Some(syn::LitStr::new("C", Span::call_site()))
    });

    let args_tokens = usr_fn.sig.inputs.iter().map(remove_mut);
    if let syn::ReturnType::Default = usr_fn.sig.output {
        if attrs.is_replace {
            return syn::Error::new(
                usr_fn.sig.ident.span(),
                "Agent frames that 'override' must specify the return type. Try adding '-> smash::lib::L2CValue'"
            ).into_compile_error().into();
        }
        usr_fn.sig.output = parse_quote! { -> smash::lib::L2CValue };
    }
    let return_tokens = usr_fn.sig.output.to_token_stream();

    let orig_name = quote::format_ident!("{}_smashline_agent_frame_orig", usr_fn_name);

    let orig_macro: syn::Stmt = parse_quote! {
        macro_rules! original {
            ($($args:expr),* $(,)?) => {
                {
                    #[allow(unused_unsafe)]
                    if true {
                        unsafe {
                            if #orig_name.is_null() {
                                panic!("Error calling agent frame {}, original function not in memory.", stringify!(#usr_fn_name));
                            } else {
                                std::mem::transmute::<_, extern "C" fn(#(#args_tokens),*) #return_tokens>(#orig_name)($($args),*)
                            }
                        }
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    };

    usr_fn.block.stmts.insert(0, orig_macro);
    if !attrs.is_replace {
        let args_names = usr_fn.sig.inputs.iter().map(get_ident);
        usr_fn.block.stmts.insert(1, parse_quote! {
            let original_result = original!(#(#args_names),*);
        });
        usr_fn.block.stmts.push(parse_quote! {
            return original_result;
        });
    }

    let install_fn = if is_fighter {
        generate_fighter_install_fn(&attrs, &usr_fn_name, &orig_name)
    } else {
        generate_weapon_install_fn(&attrs, &usr_fn_name, &orig_name)
    };

    quote!(
        #usr_fn
        
        #install_fn

        #[allow(non_upper_case_globals)]
        static mut #orig_name: *const extern "C" fn() = 0 as _;
    ).into()
}

pub fn install_agent_reset(input: TokenStream) -> TokenStream {
    let usr_fn_name = parse_macro_input!(input as syn::Ident);
    let install_name = quote::format_ident!("{}_smashline_agent_reset_install", usr_fn_name);
    quote!(
        #install_name();
    ).into()
}

pub fn agent_reset(input: TokenStream, is_fighter: bool) -> TokenStream {
    let usr_fn = parse_macro_input!(input as syn::ItemFn);

    let usr_fn_name = usr_fn.sig.ident.clone();
    let install_name = quote::format_ident!("{}_smashline_agent_reset_install", usr_fn_name);

    let install_fn = if is_fighter {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::add_fighter_reset_callback(#usr_fn_name);
                }
            }
        )
    } else {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::add_agent_reset_callback(#usr_fn_name);
                }
            }
        )
    };

    quote!(
        #usr_fn

        #install_fn
    ).into()
}

pub fn install_agent_frame_callback(input: TokenStream) -> TokenStream {
    let usr_fn_name = parse_macro_input!(input as syn::Ident);
    let install_name = quote::format_ident!("{}_smashline_agent_frame_callback_install", usr_fn_name);
    quote!(
        #install_name();
    ).into()
}

pub fn agent_frame_callback(input: TokenStream, is_fighter: bool) -> TokenStream {
    let usr_fn = parse_macro_input!(input as syn::ItemFn);
    let usr_fn_name = usr_fn.sig.ident.clone();
    let install_name = quote::format_ident!("{}_smashline_agent_frame_callback_install", usr_fn_name);

    let install_fn = if is_fighter {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::add_fighter_frame_callback(#usr_fn_name);
                }
            }
        )
    } else {
        quote!(
            #[allow(non_snake_case)]
            pub fn #install_name() {
                unsafe {
                    smashline::add_weapon_frame_callback(#usr_fn_name);
                }
            }
        )
    };

    quote!(
        #usr_fn

        #install_fn
    ).into()
}