// This code was originally written by jam1garner (https://github.com/jam1garner/) for use in skyline-acmd (https://github.com/ultimate-research/skyline-acmd)
// With his permission it has been modified and included in smashline

use proc_macro::TokenStream;
use syn::{Ident, Path, Expr, token, Token, Stmt, parse_quote, parse_macro_input, parenthesized};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::TokenStream as TokenStream2;

#[derive(Debug)]
struct AcmdFuncCall {
    pub name: Path,
    pub paren_token: syn::token::Paren,
    pub args: Punctuated<ArgExpr, Token![,]>,
    pub semi: Option<Token![;]>,
}

#[derive(Debug)]
struct ArgExpr {
    pub name: Option<Ident>,
    pub expr: Expr,
}

impl Parse for ArgExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![=]) {
            let name = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self {
                name: Some(name),
                expr: input.parse()?
            })
        } else {
            Ok(Self {
                name: None,
                expr: input.parse()?
            })
        }
    }
}

impl Parse for AcmdFuncCall {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            paren_token: syn::parenthesized!(content in input),
            args: content.parse_terminated(ArgExpr::parse)?,
            semi: if input.peek(Token![;]) {
                Some(input.parse()?)
            } else {
                None
            }
        })
    }
}

fn single_acmd_func(func_call: &AcmdFuncCall) -> TokenStream2 {
    if func_call.name.is_ident("frame") {
        // frame
        let arg = &func_call.args.iter().nth(0).expect("Missing argument in frame call").expr;
        quote!(
            ::smash::app::sv_animcmd::frame(lua_state, #arg as f32);
        )
    } else if func_call.name.is_ident("wait") {
        //wait
        let arg = &func_call.args.iter().nth(0).expect("Missing argument in wait call").expr;
        quote!(
            ::smash::app::sv_animcmd::wait(lua_state, #arg as f32);
        )
    } else if func_call.name.get_ident().is_some() {
        // ACMD functions
        let func_name = &func_call.name;
        let args = func_call.args.iter().map(|arg| arg.expr.clone());

        /* if func_name.to_token_stream().to_string() == "FT_MOTION_RATE" {
            return quote!(
                if current_frame >= target_frame {
                    ::smash::app::lua_bind::MotionModule::set_rate(
                        module_accessor,
                        #(
                            (1.0 / #args as f32).into()
                        ),*
                    );
                }
            );
        }
        else */ if func_name.to_token_stream().to_string() == "game_CaptureCutCommon" {
            return quote!(
                if current_frame >= target_frame {
                    l2c_agent.clear_lua_stack();
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(*::smash::lib::lua_const::FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(0 as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_num(3.0));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(100 as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(0 as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(60 as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_num(0.0));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_num(1.0));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(*::smash::lib::lua_const::ATTACK_LR_CHECK_F as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_num(0.0));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_bool(true));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(::smash::hash40("collision_attr_normal")));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(*::smash::lib::lua_const::ATTACK_SOUND_LEVEL_S as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(*::smash::lib::lua_const::COLLISION_SOUND_ATTR_KICK as u64));
                    l2c_agent.push_lua_stack(&mut ::smash::lib::L2CValue::new_int(*::smash::lib::lua_const::ATTACK_REGION_NONE as u64));
                    ::smash::app::sv_animcmd::ATTACK_ABS(lua_state);
                }
            );
        }

        quote!(
            l2c_agent.clear_lua_stack();
            #(
                l2c_agent.push_lua_stack(&mut (#args).into());
            )*
            ::smash::app::sv_animcmd::#func_name(lua_state);
        )
    } else if func_call.name.segments.iter().next().unwrap().ident
                    .to_string().starts_with("sv_") {
        // Lua calling convention
        let func_name = &func_call.name;
        let args = func_call.args.iter().map(|arg| arg.expr.clone());
        quote!(
            l2c_agent.clear_lua_stack();
            #(
                l2c_agent.push_lua_stack(&mut (#args).into());
            )*
            ::smash::app::#func_name(lua_state);
        )
    } else {
        // Module functions
        let func_name = &func_call.name;
        let args = func_call.args.iter().map(|arg| arg.expr.clone());
        quote!(
            ::smash::app::lua_bind::#func_name(
                module_accessor,
                #(
                    (#args).into()
                ),*
            );
        )
    }
}

pub fn generate_acmd_is_execute(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as Expr);

    if let Expr::Path(path) = expr {
        let path = path.path;
        if path.is_ident("is_execute") || path.is_ident("is_excute") {
            return quote!(
                l2c_agent.clear_lua_stack();
                ::smash::app::sv_animcmd::is_excute(lua_state);
                let #path = l2c_agent.pop_lua_stack(1).get_bool();
            ).into();
        }
    }

    quote!(

    ).into()
}

mod kw {
    syn::custom_keyword!(rust);
    syn::custom_keyword!(Iterations);
}

struct AcmdBlock {
    pub braces: syn::token::Brace,
    pub statements: Vec<AcmdStatement>
}

impl Parse for AcmdBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            braces: syn::braced!(content in input),
            statements: {
                let mut items = Vec::new();
                while !content.is_empty() {
                    items.push(content.parse()?);
                }
                items
            }
        })
    }
}

impl ToTokens for AcmdBlock {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append_all(self.statements.iter())
    }
}

struct AcmdIf {
    pub if_token: Token![if],
    pub parens: syn::token::Paren,
    pub cond: Expr,
    pub block: AcmdBlock,
    pub else_token: Option<Token![else]>,
    pub else_block: Option<AcmdBlock>,
}

impl Parse for AcmdIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let mut acmd_if = Self {
            if_token: input.parse()?,
            parens: syn::parenthesized!(content in input),
            cond: content.parse()?,
            block: input.parse()?,
            else_token: None,
            else_block: None
        };

        let lookahead = input.lookahead1();

        if lookahead.peek(Token![else]) {
            acmd_if.else_token = Some(input.parse()?);
            acmd_if.else_block = Some(input.parse()?);
        }

        Ok(acmd_if)
    }
}

struct AcmdFor {
    pub for_token: Token![for],
    pub parens: syn::token::Paren,
    pub iter_count: Expr,
    pub iter_keyword: kw::Iterations,
    pub block: AcmdBlock
}

impl Parse for AcmdFor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            for_token: input.parse()?,
            parens: syn::parenthesized!(content in input),
            iter_count: content.parse()?,
            iter_keyword: content.parse()?,
            block: input.parse()?
        })
    }
}

struct InlineRustBlock {
    pub rust_token: kw::rust,
    pub block: syn::Block
}

impl Parse for InlineRustBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            rust_token: input.parse()?,
            block: input.parse()?
        })
    }
}

enum AcmdStatement {
    If(AcmdIf),
    For(AcmdFor),
    FuncCall(AcmdFuncCall),
    RustBlock(InlineRustBlock),
}

impl Parse for AcmdStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            Ok(Self::If(input.parse()?))
        } else if lookahead.peek(Token![for]) {
            Ok(Self::For(input.parse()?))
        } else if lookahead.peek(kw::rust) {
            Ok(Self::RustBlock(input.parse()?))
        } else {
            Ok(Self::FuncCall(input.parse()?))
        }
    }
}

impl ToTokens for AcmdStatement {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let new_tokens = match self {
            Self::If(acmd_if) => {
                let default_block = AcmdBlock { braces: syn::token::Brace::default(), statements: Vec::with_capacity(0) };
                let cond = &acmd_if.cond;
                let acmd_block = &acmd_if.block;
                let else_block = &acmd_if.else_block.as_ref().unwrap_or(&default_block);
                quote!(
                    ::smashline::generate_acmd_is_execute!(#cond);
                    if #cond {
                        #acmd_block
                    }
                    else {
                        #else_block
                    }
                )
            }
            Self::For(acmd_for) => {
                let iter_count = &acmd_for.iter_count;
                let acmd_block = &acmd_for.block;
                quote!(
                    for _ in (0..#iter_count) {
                        #acmd_block
                    }
                )
            }
            Self::FuncCall(func_call) => {
                single_acmd_func(func_call)
            }
            Self::RustBlock(rust_block) => {
                let stmts = rust_block.block.stmts.iter();
                quote!(
                    #(
                        #stmts
                    )*
                )
            }
        };
        tokens.append_all([new_tokens].iter());
    }
}

struct AcmdInput {
    pub l2c_state: Option<Expr>,
    pub acmd: AcmdBlock
}

impl Parse for AcmdInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(Token![,]) {
            let l2c_state = Some(input.parse()?);
            let _: Token![,] = input.parse()?;
            Ok(Self {
                l2c_state,
                acmd: input.parse()?
            })
        } else {
            Ok(Self {
                l2c_state: None,
                acmd: input.parse()?
            })
        }
    }
}

pub fn acmd(input: TokenStream) -> TokenStream {
    let acmd_input = syn::parse_macro_input!(input as AcmdInput);

    let setup = acmd_input.l2c_state.map(|l2c_state|{
        quote!(
            let l2c_agent = &mut ::smash::lib::L2CAgent::new(#l2c_state);
            let lua_state = #l2c_state;
            let module_accessor = ::smash::app::sv_system::battle_object_module_accessor(lua_state);
        )
    });

    let acmd_stmts = acmd_input.acmd.statements;

    quote!(
        unsafe {
            #setup
    
            #(
                #acmd_stmts
            )*
        }
    ).into()
}