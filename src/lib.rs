// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type = "proc-macro"]
#![doc(html_root_url = "https://docs.rs/num-derive/0.1")]

extern crate proc_macro;

extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;

use syn::{Data, Fields, Ident};

#[proc_macro_derive(FromPrimitive)]
pub fn from_primitive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let dummy_const = Ident::new(&format!("_IMPL_NUM_FROM_PRIMITIVE_FOR_{}", name), Span::call_site());

    let variants = match ast.data {
        Data::Enum(ref data_enum) => &data_enum.variants,
        _ => panic!("`FromPrimitive` can be applied only to the enums, {} is not an enum", name)
    };

    let from_i64_var = quote! { n };
    let mut expr = quote! { 0i64 };
    let mut offset = 0i64;
    let clauses: Vec<_> = variants.iter()
        .map(|variant| {
            let ident = &variant.ident;
            match variant.fields {
                Fields::Unit => (),
                _ => {
                    panic!("`FromPrimitive` can be applied only to unitary enums, {}::{} is either struct or tuple", name, ident)
                },
            }

            let discriminant_expr = match variant.discriminant {
                Some((_, ref const_expr)) => {
                    expr = quote! { (#const_expr) as i64 };
                    offset = 1;
                    expr.clone()
                }
                None => {
                    let tt = quote! { #expr + #offset };
                    offset += 1;
                    tt
                }
            };

            quote! {
                if #from_i64_var == #discriminant_expr {
                    Some(#name::#ident)
                }
            }
        })
        .collect();

    let from_i64_var = if clauses.is_empty() { quote!(_) } else { from_i64_var };

    let res = quote! {
        #[allow(non_upper_case_globals)]
        #[allow(unused_qualifications)]
        const #dummy_const: () = {
            extern crate num as _num;

            impl _num::traits::FromPrimitive for #name {
                #[allow(trivial_numeric_casts)]
                fn from_i64(#from_i64_var: i64) -> Option<Self> {
                    #(#clauses else)* {
                        None
                    }
                }

                fn from_u64(n: u64) -> Option<Self> {
                    Self::from_i64(n as i64)
                }
            }
        };
    };

    res.into()
}

#[proc_macro_derive(ToPrimitive)]
pub fn to_primitive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let dummy_const = Ident::new(&format!("_IMPL_NUM_TO_PRIMITIVE_FOR_{}", name), Span::call_site());

    let variants = match ast.data {
        Data::Enum(ref data_enum) => &data_enum.variants,
        _ => panic!("`ToPrimitive` can be applied only to the enums, {} is not an enum", name)
    };

    let mut expr = quote! { 0i64 };
    let mut offset = 0i64;
    let variants: Vec<_> = variants.iter()
        .map(|variant| {
            let ident = &variant.ident;
            match variant.fields {
                Fields::Unit => (),
                _ => {
                    panic!("`ToPrimitive` can be applied only to unitary enums, {}::{} is either struct or tuple", name, ident)
                },
            }

            let discriminant_expr = match variant.discriminant {
                Some((_, ref const_expr)) => {
                    expr = quote! { (#const_expr) as i64 };
                    offset = 1;
                    expr.clone()
                }
                None => {
                    let tt = quote! { #expr + #offset };
                    offset += 1;
                    tt
                }
            };

            quote!(#name::#ident => (#discriminant_expr))
        })
        .collect();

    let match_expr = if variants.is_empty() {
        // No variants found, so do not use Some to not to trigger `unreachable_code` lint
        quote! {
            match *self {}
        }
    } else {
        quote! {
            Some(match *self {
                #(#variants,)*
            })
        }
    };

    let res = quote! {
        #[allow(non_upper_case_globals)]
        #[allow(unused_qualifications)]
        const #dummy_const: () = {
            extern crate num as _num;

            impl _num::traits::ToPrimitive for #name {
                #[allow(trivial_numeric_casts)]
                fn to_i64(&self) -> Option<i64> {
                    #match_expr
                }

                fn to_u64(&self) -> Option<u64> {
                    self.to_i64().map(|x| x as u64)
                }
            }
        };
    };

    res.into()
}
