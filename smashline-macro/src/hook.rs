use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, token, Token};

use crate::attrs::*;
use crate::remove_mut;

use owo_colors::OwoColorize;

fn generate_install_fn(
    module: &HookModule,
    symbol: &HookSymbol,
    usr_fn: &syn::Ident,
    orig_fn: &syn::Ident,
) -> impl ToTokens {
    let install_fn = quote::format_ident!("{}_smashline_hook_install", usr_fn);

    if let HookModule::Lazy(module) = module {
        if let HookSymbol::Resolved(_symbol) = symbol {
            syn::Error::new(
                module.span(),
                "Lazy (module) hooks cannot use resolved symbols.",
            )
            .into_compile_error()
        } else if let HookSymbol::Unresolved(symbol) = symbol {
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
    let install_fn = quote::format_ident!("{}_smashline_hook_install", name);
    quote!(
        unsafe { #install_fn(); }
    )
    .into()
}

pub fn generate_hook_fn(attrs: &HookAttrs, mut replacement_fn: syn::ItemFn) -> TokenStream2 {
    let mut output = TokenStream2::new();

    let HookAttrs { module, symbol } = attrs;

    if let HookModule::Lazy(module) = &module {
        if let HookSymbol::Resolved(_symbol) = &symbol {
            return syn::Error::new(
                module.span(),
                "Lazy (module) hooks cannot use resolved symbols.",
            )
            .into_compile_error()
            .into();
        }
    }

    // extern "C"
    replacement_fn.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern {
            span: Span::call_site(),
        },
        name: Some(syn::LitStr::new("C", Span::call_site())),
    });

    let args_tokens = replacement_fn.sig.inputs.iter().map(remove_mut);
    let return_tokens = replacement_fn.sig.output.to_token_stream();

    let usr_fn = replacement_fn.sig.ident.clone();

    let orig_fn = quote::format_ident!("{}_smashline_hook_orig", usr_fn);

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

    quote!(
        #replacement_fn

        #[allow(non_upper_case_globals)]
        pub static mut #orig_fn: *const extern "C" fn() = 0 as _;
    )
    .to_tokens(&mut output);

    output.into()
}

pub fn hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let replacement_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = parse_macro_input!(attrs as HookAttrs);

    let usr_fn = replacement_fn.sig.ident.clone();

    let orig_fn = quote::format_ident!("{}_smashline_hook_orig", usr_fn);

    let without_install = generate_hook_fn(&attrs, replacement_fn);

    let install_fn = generate_install_fn(&attrs.module, &attrs.symbol, &usr_fn, &orig_fn);

    quote!(
        #without_install

        #install_fn
    )
    .into()
}

pub fn raw_inline_hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as RawHook);
    let mut replacement_fn = parse_macro_input!(input as syn::ItemFn);

    let RawHook { feature, hook_args } = attrs;
    let feature = if let Some(feature) = feature {
        feature
    } else {
        syn::parse_quote!("runtime")
    };

    let runtime_fn_name = quote::format_ident!("{}_impl", replacement_fn.sig.ident);
    let static_fn_name = quote::format_ident!("{}_bootstrap", replacement_fn.sig.ident);
    replacement_fn.sig.ident = runtime_fn_name;
    let mut runtime_sig = replacement_fn.sig.clone();
    runtime_sig.unsafety = None;
    replacement_fn.sig.abi = Some(syn::parse_quote!(extern "C"));
    replacement_fn.vis = parse_quote!(pub);
    let runtime_fn_name = &runtime_sig.ident;
    let runtime_symbol =
        syn::LitStr::new(runtime_sig.ident.to_string().as_str(), Span::call_site());

    let mut static_sig = runtime_sig.clone();
    static_sig.ident = static_fn_name;

    let args = replacement_fn.sig.inputs.iter().map(|arg| match arg {
        syn::FnArg::Typed(syn::PatType { pat, .. }) => quote!(#pat),
        _ => quote!(),
    });

    let args1 = args.clone();
    let args2 = args.clone();

    quote! {
        #[cfg(all(feature = "development", feature = #feature))]
        #[export_name = #runtime_symbol]
        #replacement_fn

        #[cfg(not(feature = "development"))]
        #[skyline::hook(#hook_args)]
        #static_sig {
            #runtime_fn_name(#(#args1,)*)
        }

        #[cfg(all(feature = "development", not(feature = #feature)))]
        #[skyline::hook(#hook_args)]
        #[allow(unused_unsafe)]
        #static_sig {
            extern "C" {
                #[link_name = #runtime_symbol]
                #runtime_sig;
            }

            unsafe {
                if crate::is_development_installed() {
                    #runtime_fn_name(#(#args2,)*)
                }
            }

        }
    }
    .into()
}

pub fn raw_hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as RawHook);
    let mut replacement_fn = parse_macro_input!(input as syn::ItemFn);

    let RawHook { feature, hook_args } = attrs;
    let feature = if let Some(feature) = feature {
        feature
    } else {
        syn::parse_quote!("runtime")
    };

    let runtime_fn_name = quote::format_ident!("{}_impl", replacement_fn.sig.ident);
    let static_fn_name = quote::format_ident!("{}_bootstrap", replacement_fn.sig.ident);
    replacement_fn.sig.ident = runtime_fn_name;
    let mut runtime_sig = replacement_fn.sig.clone();
    runtime_sig.unsafety = None;
    replacement_fn.sig.abi = Some(syn::parse_quote!(extern "C"));
    replacement_fn.vis = parse_quote!(pub);
    let runtime_fn_name = &runtime_sig.ident;
    let runtime_symbol =
        syn::LitStr::new(runtime_sig.ident.to_string().as_str(), Span::call_site());

    let mut static_sig = runtime_sig.clone();
    static_sig.ident = static_fn_name;

    let fn_type_args = replacement_fn.sig.inputs.iter().map(remove_mut);
    let ret_ty = &replacement_fn.sig.output;

    let args: Vec<TokenStream2> = replacement_fn
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(syn::PatType { pat, .. }) => quote!(#pat),
            _ => quote!(),
        })
        .collect();

    let args1 = args.iter();
    let args2 = args.iter();
    let args3 = args.iter();

    let fn_arg: syn::FnArg =
        syn::parse_quote!(original_fn_: extern "C" fn(#(#fn_type_args,)*) #ret_ty);
    replacement_fn.sig.inputs.push(fn_arg.clone());

    runtime_sig.inputs.push(fn_arg);

    replacement_fn.block.stmts.insert(
        0,
        syn::parse_quote!(
            macro_rules! call_original {
                ($($args:expr),* $(,)?) => {{
                    original!()($($args),*)
                }}
            }
        ),
    );

    replacement_fn.block.stmts.insert(
        0,
        syn::parse_quote!(
            macro_rules! original {
                () => {{
                    original_fn_
                }};
            }
        ),
    );

    quote! {
        #[cfg(all(feature = "development", feature = #feature))]
        #[export_name = #runtime_symbol]
        #replacement_fn

        #[cfg(not(feature = "development"))]
        #[skyline::hook(#hook_args)]
        #static_sig {
            #runtime_fn_name(#(#args1,)* original!())
        }

        #[cfg(all(feature = "development", not(feature = #feature)))]
        #[skyline::hook(#hook_args)]
        #[allow(unused_unsafe)]
        #static_sig {
            extern "C" {
                #[link_name = #runtime_symbol]
                #runtime_sig;
            }

            unsafe {
                if crate::is_development_installed() {
                    #runtime_fn_name(#(#args2,)* original!())
                } else {
                    call_original!(#(#args3,)*)
                }
            }

        }
    }
    .into()
}

pub fn development_state_tracker(input: TokenStream) -> TokenStream {
    let StateTracker { feature } = syn::parse_macro_input!(input);

    let feature = if let Some(feature) = feature {
        feature
    } else {
        syn::parse_quote!("runtime")
    };

    quote! {
        #[cfg(all(feature = "development", not(feature = #feature)))]
        static mut RUNTIME_PLUGIN_INSTALLED: bool = false;

        #[cfg(all(feature = "development", feature = #feature))]
        extern "C" {
            #[link_name = "set_dev_plugin_available"]
            pub fn set_dev_plugin_available(arg: bool);
        }

        #[cfg(all(feature = "development", not(feature = #feature)))]
        #[export_name = "set_dev_plugin_available"]
        pub extern "C" fn set_dev_plugin_available(arg: bool) {
            unsafe {
                RUNTIME_PLUGIN_INSTALLED = arg;
            }
        }

        #[cfg(all(feature = "development", not(feature = #feature)))]
        pub fn is_development_installed() -> bool {
            unsafe {
                RUNTIME_PLUGIN_INSTALLED
            }
        }
    }
    .into()
}

pub fn install_raw_hook(input: TokenStream) -> TokenStream {
    let RawHook { feature, hook_args } = parse_macro_input!(input);

    let feature = if let Some(feature) = feature {
        feature
    } else {
        parse_quote!("runtime")
    };

    let hook_name: syn::Ident = match syn::parse2(hook_args) {
        Ok(id) => id,
        Err(e) => return e.into_compile_error().into(),
    };

    let hook_name = quote::format_ident!("{}_bootstrap", hook_name);

    quote! {
        #[cfg(all(feature = "development", not(feature = #feature)))]
        {
            skyline::install_hook!(#hook_name);
        }
    }
    .into()
}
