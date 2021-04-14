#![feature(asm)]
#![feature(const_loop)]
#![feature(const_if_match)]
use syn::{parse_quote, parse_macro_input, Token, token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use owo_colors::OwoColorize;

mod kw {
    syn::custom_keyword!(module);
    syn::custom_keyword!(symbol);
}


enum HookModule {
    Lazy(syn::LitStr),
    Static(token::Static)
}

impl Parse for HookModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(module_name) = input.parse::<syn::LitStr>() {
            Ok(HookModule::Lazy(module_name))
        } else {
            let static_kw = input.parse()?;
            Ok(HookModule::Static(static_kw))
        }
    }
}

enum HookSymbol {
    Resolved(syn::Path),
    Unresolved(syn::LitStr)
}

impl Parse for HookSymbol {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(symbol) = input.parse::<syn::LitStr>() {
            Ok(HookSymbol::Unresolved(symbol))
        } else {
            let symbol = input.parse()?;
            Ok(HookSymbol::Resolved(symbol))
        }
    }
}

struct HookAttrs {
    pub module: HookModule,
    pub symbol: HookSymbol
}

impl Parse for HookAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let module = if input.peek(kw::module) {
            let MetaItem::<kw::module, HookModule> { item: module_name, .. } = input.parse()?;

            Ok(module_name)
        } else {
            Err(input.error(format!(
                "Expected keyword '{}' in macro declaration.", "module".bright_blue()
            )))
        }?;

        let _: syn::Token![,] = input.parse()?;

        let symbol = if input.peek(kw::symbol) {
            let MetaItem::<kw::symbol, HookSymbol> { item: symbol, .. } = input.parse()?;
            Ok(symbol)
        } else {
            Err(input.error(format!(
                "Expected keyword '{}' in macro declaration.", "symbol".bright_blue()
            )))
        }?;

        Ok(HookAttrs {
            module,
            symbol
        })
    }
}

// taken from skyline-rs hooking implementation
struct MetaItem<Keyword: Parse, Item: Parse> {
    pub ident: Keyword,
    pub item: Item
}

impl<Keyword: Parse, Item: Parse> Parse for MetaItem<Keyword, Item> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let item = if input.peek(token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            content.parse()?
        } else if input.peek(token::Bracket)  {
            let content;
            syn::bracketed!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(Self {
            ident,
            item
        })
    }
}

fn remove_mut(arg: &syn::FnArg) -> syn::FnArg {
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

fn generate_install_fn(module: &HookModule, symbol: &HookSymbol, usr_fn: &syn::Ident, orig_fn: &syn::Ident) -> impl ToTokens {
    let install_fn = quote::format_ident!(
        "{}_smashline_hook_install_fn", usr_fn
    );

    if let HookModule::Lazy(module) = module {
        if let HookSymbol::Resolved(symbol) = symbol {
            syn::Error::new(module.span(), "Lazy (module) hooks cannot use resolved symbols.").into_compile_error()
        } else if let HookSymbol::Unresolved(symbol) = symbol{
            quote! {
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
    } else if let HookModule::Static(module) = module {
        if let HookSymbol::Resolved(symbol) = symbol {
            quote! {
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

#[proc_macro]
pub fn install_hook(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);
    let install_fn = quote::format_ident!(
        "{}_smashline_hook_install_fn", name
    );
    quote!(
        unsafe { #install_fn(); }
    ).into()
}

#[proc_macro_attribute]
pub fn hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut replacement_fn = parse_macro_input!(input as syn::ItemFn);
    let HookAttrs { module, symbol } = parse_macro_input!(attrs as HookAttrs);
    let mut output = TokenStream2::new();

    if let HookModule::Lazy(ref module) = &module {
        if let HookSymbol::Resolved(ref symbol) = &symbol {
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
        "{}_smashline_hook_original_fn", usr_fn
    );

    let usr_fn_str = syn::LitStr::new(usr_fn.to_string().as_str(), Span::call_site());

    // allow for original!() and call_original! like in skyline-rs
    let orig_macro: syn::Stmt = parse_quote! {
        macro_rules! original {
            () => {
                {
                    #[allow(unused_unsafe)]
                    if true {
                        unsafe {
                            if #orig_fn.is_null() {
                                panic!("Error calling function hook {}, original function not in memory.", #usr_fn_str);
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