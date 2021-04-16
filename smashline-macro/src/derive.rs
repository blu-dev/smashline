use syn::{parse_quote, parse_macro_input, Token, token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

pub fn derive_lua_struct(item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as syn::DeriveInput);
    let structure = synstructure::Structure::new(&item_struct);

    let struct_name = &structure.ast().ident;
    
    let mut into_l2cvalue: syn::ItemFn = parse_quote! {
        fn into(self) -> smash::lib::L2CValue {
            let table = smash::lib::L2CTable::new(0);
            let mut ret = smash::lib::L2CValue::Table(table);
        }
    };

    let mut from_l2cvalue: syn::ItemFn = parse_quote! {
        fn from(val: &smash::lib::L2CValue) -> Self {
            assert!(val.val_type == smash::lib::L2CValueType::Table);
        }
    };

    let mut write_l2cvalue: syn::ItemFn = parse_quote! {
        fn write_value(&self, val: &mut L2CValue) {
            assert!(val.val_type == smash::lib::L2CValueType::Table);
        }
    };

    let mut from_return_struct = TokenStream2::new();

    let _ = structure.each(|bi| {
        let ident = bi.ast().ident.as_ref().unwrap();

        let stmt: syn::Stmt = parse_quote! {
            ret[stringify!(#ident)] = self.#ident.clone().into();
        };
        into_l2cvalue.block.stmts.push(stmt);

        let stmt: syn::Stmt = parse_quote! {
            let #ident = (&val[stringify!(#ident)]).into();
        };
        from_l2cvalue.block.stmts.push(stmt);

        let stmt: syn::Stmt = parse_quote! {
            val[stringify!(#ident)] = self.#ident.clone().into();
        };

        write_l2cvalue.block.stmts.push(stmt);

        quote! (
            #ident, 
        ).to_tokens(&mut from_return_struct);
        quote!()
    });
    into_l2cvalue.block.stmts.push(parse_quote! { return ret; });

    let from_return_stmt = parse_quote! {
        return Self { #from_return_struct };
    };
    from_l2cvalue.block.stmts.push(from_return_stmt);

    quote!(
        impl Into<smash::lib::L2CValue> for #struct_name {
            #into_l2cvalue
        }

        impl From<&smash::lib::L2CValue> for #struct_name {
            #from_l2cvalue
        }

        impl #struct_name {
            #write_l2cvalue
        }
    ).into()
}