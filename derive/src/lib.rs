#![recursion_limit = "4096"]

extern crate proc_macro;

use quote::quote;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, PathArguments, GenericArgument,
    FieldsUnnamed, Ident, Meta, Field, MetaList, NestedMeta, Type,
};

#[proc_macro_derive(Classicalize, attributes(prost))]
pub fn classicalize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let s = match input.data {
        Data::Struct(s) => classicalize_struct(input.ident, s),
        Data::Enum(e) => classicalize_enum(input.ident, e),
        Data::Union(_) => panic!("union is not supported yet."),
    };
    s.into()
}

fn classicalize_accessors(field: &Field) -> Option<proc_macro2::TokenStream> {
    let is_message = field.attrs.iter().any(|a| {
        a.interpret_meta().iter().any(|m| {
            match m {
                Meta::List(MetaList { ident, nested, .. }) => {
                    ident == "prost" && 
                        nested.iter().any(|a| match a {
                            NestedMeta::Meta(Meta::Word(ty)) => ty == "message",
                            _ => false
                        })
                }
                _ => false,
            }
        })
    });
    if !is_message {
        return None;
    }
    let ident = field.ident.as_ref().unwrap();
    let mut ident_str = ident.to_string();
    if ident_str.starts_with("r#") {
        ident_str = ident_str[2..].to_owned();
    }
    let ty = &field.ty;
    let ty = match ty {
        Type::Path(tp) => {
            let wrapper = tp.path.segments.iter().last().unwrap();
            assert_eq!(wrapper.ident, "Option", "expected option, but got {:?}", ty);
            let generic_arg = match wrapper.arguments {
                PathArguments::AngleBracketed(ref params) => params.args.iter().next().unwrap(),
                _ => panic!("unexpected token {:?}", ty),
            };
            match generic_arg {
                GenericArgument::Type(ty) => ty,
                _ => panic!("expected generic, but get {:?}", ty),
            }
        },
        _ => panic!("unexpected type {:?}", ty),
    };
    let set = Ident::new(&format!("set_{}", ident_str), Span::call_site());
    let get = Ident::new(&format!("get_{}", ident_str), Span::call_site());
    let mutation = Ident::new(&format!("mut_{}", ident_str), Span::call_site());
    Some(quote! {
        pub fn #set(&mut self, value: #ty) {
            self.#ident = Some(value);
        }

        pub fn #get(&self) -> &#ty {
            self.#ident.as_ref().unwrap_or_else(|| #ty::default_instance())
        }

        pub fn #mutation(&mut self) -> &mut #ty {
            self.#ident.get_or_insert_with(|| #ty::default())
        }
    })
}

fn classicalize_struct(ident: Ident, s: DataStruct) -> proc_macro2::TokenStream {
    let fields = match s {
        DataStruct {
            fields: Fields::Named(FieldsNamed { named: fields, .. }),
            ..
        }
        | DataStruct {
            fields:
                Fields::Unnamed(FieldsUnnamed {
                    unnamed: fields, ..
                }),
            ..
        } => fields.into_iter().collect(),
        DataStruct {
            fields: Fields::Unit,
            ..
        } => Vec::new(),
    };

    // Put impls in a const, so that 'extern crate' can be used.
    let dummy_const = Ident::new(&format!("{}_CLASSICAL_MESSAGE", ident), Span::call_site());

    let methods = fields
        .iter()
        .flat_map(classicalize_accessors)
        .collect::<Vec<_>>();
    let methods = if methods.is_empty() {
        quote!()
    } else {
        quote! {
            #[allow(dead_code)]
            impl #ident {
                #(#methods)*
            }
        }
    };

    quote! {
        #[allow(non_snake_case, unused_attributes)]
        const #dummy_const: () = {
            extern crate prost as _prost;
            extern crate bytes as _bytes;
            extern crate lazy_static;

            impl #ident {
                pub fn default_instance() -> &'static #ident {
                    lazy_static::lazy_static! {
                        static ref INSTANCE: #ident = #ident::default();
                    }
                    &*INSTANCE
                }
            }

            #methods
        };
    }
}

fn classicalize_enum(ident: Ident, s: DataEnum) -> proc_macro2::TokenStream {
    let dummy_const = Ident::new(&format!("{}_CLASSICAL_ENUMERATION", ident), Span::call_site());

    // Map the variants into 'fields'.
    let mut variants = Vec::with_capacity(s.variants.len());
    for v in s.variants {
        let value_ident = &v.ident;
        variants.push(quote! { #ident::#value_ident});
    }
    quote! {
        #[allow(non_snake_case, unused_attributes)]
        const #dummy_const: () = {
            extern crate jinkela as _jinkela;

            impl _jinkela::GenericEnum for #ident {
                fn values() -> &'static [#ident] {
                    &[#(#variants,)*]
                }
            }
        };
    }
}
