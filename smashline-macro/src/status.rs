use syn::{parse_quote, parse_macro_input, Token, token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use crate::attrs::*;
use crate::remove_mut;

fn generate_install_fn(attrs: &StatusAttrs, usr_fn_name: &syn::Ident, orig_name: &syn::Ident) -> impl ToTokens {
    let install_name = quote::format_ident!("{}_smashline_status_script_install", usr_fn_name);

    let agent = &attrs.agent;
    let status = &attrs.status;
    let condition = &attrs.condition;
    let low_priority = &attrs.low_priority;

    quote!(
        #[allow(non_snake_case)]
        #[allow(unused_unsafe)]
        pub fn #install_name() {
            unsafe {
                smashline::replace_status_script(#agent, #status, #condition, Some(&mut #orig_name), #low_priority, #usr_fn_name as *const extern "C" fn());
            }
        }
    )
}

pub fn install_status_script(input: TokenStream) -> TokenStream {
    let usr_fn_name = parse_macro_input!(input as syn::Ident);
    let install_name = quote::format_ident!("{}_smashline_status_script_install", usr_fn_name);

    quote!(
        unsafe { #install_name() };
    ).into()
}

pub fn status_script(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as StatusAttrs);
    let mut usr_fn = parse_macro_input!(input as syn::ItemFn);

    let usr_fn_name = usr_fn.sig.ident.clone();

    usr_fn.sig.abi = Some(syn::Abi {
        extern_token: token::Extern { span: Span::call_site() },
        name: Some(syn::LitStr::new("C", Span::call_site()))
    });

    let args_tokens = usr_fn.sig.inputs.iter().map(remove_mut);
    let return_tokens = usr_fn.sig.output.to_token_stream();

    let orig_name = quote::format_ident!("{}_smashline_status_script_orig", usr_fn_name);

    let orig_macro: syn::Stmt = parse_quote! {
        macro_rules! original {
            ($($args:expr),* $(,)?) => {
                {
                    #[allow(unused_unsafe)]
                    if true {
                        unsafe {
                            if #orig_name.is_null() {
                                panic!("Error calling function hook {}, original function not in memory.", stringify!(#usr_fn_name));
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

    let install_fn = generate_install_fn(&attrs, &usr_fn_name, &orig_name);

    quote!(
        #usr_fn

        #install_fn

        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        static mut #orig_name: *const extern "C" fn() = 0 as _;
    ).into()
}

pub fn common_status_script(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as CommonStatusAttrs);
    let input = parse_macro_input!(input as syn::ItemFn);


    let hook_attrs = HookAttrs {
        module: HookModule::Lazy(syn::LitStr::new("common", Span::call_site())),
        symbol: HookSymbol::Unresolved(attrs.symbol.unwrap_or(syn::LitStr::new("", Span::call_site())))
    };

    let usr_fn_name = input.sig.ident.clone();
    let install_name = quote::format_ident!("{}_smashline_status_script_install", usr_fn_name);

    let orig_name = quote::format_ident!("{}_smashline_hook_orig", usr_fn_name);

    let status = &attrs.status;
    let condition = &attrs.condition;

    let hook_fn = crate::hook::generate_hook_fn(&hook_attrs, input);
    if let HookSymbol::Unresolved(symbol) = &hook_attrs.symbol {
        let install_fn = quote!(
            pub fn #install_name() {
                unsafe {
                    if #symbol != "" {
                        smashline::replace_symbol("common", #symbol, #usr_fn_name as *const extern "C" fn(), Some(&mut #orig_name));
                        smashline::replace_common_status_script(#status, #condition, None, #usr_fn_name as *const extern "C" fn());
                    } else {
                        smashline::replace_common_status_script(#status, #condition, Some(&mut #orig_name), #usr_fn_name as *const extern "C" fn());
                    }
                }
            }
        );

        quote!(
            #install_fn
            #hook_fn
        ).into()
    } else {
        unreachable!()
    }

}