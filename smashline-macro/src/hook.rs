use syn::{parse_quote, parse_macro_input, Token, token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use crate::attrs::*;
use crate::remove_mut;

use owo_colors::OwoColorize;

fn generate_install_fn(module: &HookModule, symbol: &HookSymbol, usr_fn: &syn::Ident, orig_fn: &syn::Ident) -> impl ToTokens {
    let install_fn = quote::format_ident!(
        "{}_smashline_hook_install", usr_fn
    );

    if let HookModule::Lazy(module) = module {
        if let HookSymbol::Resolved(_symbol) = symbol {
            syn::Error::new(module.span(), "Lazy (module) hooks cannot use resolved symbols.").into_compile_error()
        } else if let HookSymbol::Unresolved(symbol) = symbol{
            quote! {
                #[allow(non_snake_case)]
                #[allow(unused_unsafe)]
                pub fn #install_fn() {
                    unsafe {
                        if (smashline::replace_symbol as *const ()).is_null() {
                            panic!("smashline::replace_symbol is missing -- maybe missing libsmashline_hook.nro?");
                        }
                        smashline::replace_symbol(#module, #symbol, #usr_fn as *const extern "C" fn(), Some(&mut #orig_fn));
                    }
                }
            }
        } else {
            unreachable!()
        }
    } else if let HookModule::Static(_) = module {
        if let HookSymbol::Resolved(symbol) = symbol {
            quote! {
                #[allow(non_snake_case)]
                #[allow(unused_unsafe)]
                pub fn #install_fn() {
                    unsafe {
                        if (smashline::replace_static_symbol as *const ()).is_null() {
                            panic!("smashline::replace_static_symbol is missing -- maybe missing libsmashline_hook.nro?");
                        }
                        smashline::replace_static_symbol(smashline::StaticSymbol::Resolved(#symbol as *const () as usize), #usr_fn as *const extern "C" fn(), Some(&mut #orig_fn));
                    }
                }
            }
        } else if let HookSymbol::Unresolved(symbol) = symbol {
            quote! {
                #[allow(non_snake_case)]
                #[allow(unused_unsafe)]
                pub fn #install_fn() {
                    unsafe {
                        if (smashline::replace_static_symbol as *const ()).is_null() {
                            panic!("smashline::replace_static_symbol is missing -- maybe missing libsmashline_hook.nro?");
                        }
                        smashline::replace_static_symbol(smashline::StaticSymbol::Unresolved(#symbol), #usr_fn as *const extern "C" fn(), Some(&mut #orig_fn));
                    }
                }
            }
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

pub fn install_hook(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);
    let install_fn = quote::format_ident!(
        "{}_smashline_hook_install", name
    );
    quote!(
        unsafe { #install_fn(); }
    ).into()
}

pub fn hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut replacement_fn = parse_macro_input!(input as syn::ItemFn);
    let HookAttrs { module, symbol } = parse_macro_input!(attrs as HookAttrs);
    let mut output = TokenStream2::new();

    if let HookModule::Lazy(module) = &module {
        if let HookSymbol::Resolved(_symbol) = &symbol {
            return syn::Error::new(module.span(), "Lazy (module) hooks cannot use resolved symbols.").into_compile_error().into();
        }
    }

    // extern "C"
    replacement_fn.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern { span: Span::call_site() },
        name: Some(syn::LitStr::new("C", Span::call_site()))
    });

    let args_tokens = replacement_fn.sig.inputs.iter().map(remove_mut);
    let return_tokens = replacement_fn.sig.output.to_token_stream();

    let usr_fn = replacement_fn.sig.ident.clone();

    let orig_fn = quote::format_ident!(
        "{}_smashline_hook_orig", usr_fn
    );

    // allow for original!() and call_original! like in skyline-rs
    let orig_macro: syn::Stmt = parse_quote! {
        macro_rules! original {
            () => {
                {
                    #[allow(unused_unsafe)]
                    if true {
                        unsafe {
                            if #orig_fn.is_null() {
                                panic!("Error calling function hook {}, original function not in memory.", stringify!(#usr_fn));
                            } else {
                                std::mem::transmute::<_, extern "C" fn(#(#args_tokens),*) #return_tokens>(
                                    #orig_fn as *const()
                                )
                            }
                        }
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    };

    replacement_fn.block.stmts.insert(0, orig_macro);
    let orig_macro: syn::Stmt = parse_quote! {
        macro_rules! call_original {
            ($($args:expr),* $(,)?) => {
                original!()($($args),*)
            }
        }
    };
    replacement_fn.block.stmts.insert(1, orig_macro);

    replacement_fn.to_tokens(&mut output);

    let install_fn = generate_install_fn(&module, &symbol, &usr_fn, &orig_fn);

    quote!(
        #install_fn

        #[allow(non_upper_case_globals)]
        pub static mut #orig_fn: *const extern "C" fn() = 0 as _;
    ).to_tokens(&mut output);

    output.into()
}