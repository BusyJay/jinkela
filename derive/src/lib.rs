#![recursion_limit = "4096"]

extern crate proc_macro;

use quote::quote;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, PathArguments, GenericArgument,
    FieldsUnnamed, Ident, Meta, Field, MetaList, NestedMeta, Type, Lit, Path,
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

fn classicalize_optional_message_field(field: &Field) -> proc_macro2::TokenStream {
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
    let take = Ident::new(&format!("take_{}", ident_str), Span::call_site());
    let mutation = Ident::new(&format!("mut_{}", ident_str), Span::call_site());
    quote! {
        pub fn #set(&mut self, value: #ty) {
            self.#ident = Some(value);
        }

        pub fn #get(&self) -> &#ty {
            self.#ident.as_ref().unwrap_or_else(|| #ty::default_instance())
        }

        pub fn #mutation(&mut self) -> &mut #ty {
            self.#ident.get_or_insert_with(|| #ty::default())
        }

        pub fn #take(&mut self) -> #ty {
            self.#ident.take().unwrap_or_else(|| #ty::default())
        }
    }
}

fn classicalize_repeated_field(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let mut ident_str = ident.to_string();
    if ident_str.starts_with("r#") {
        ident_str = ident_str[2..].to_owned();
    }
    let origin_ty = &field.ty;
    let ty = match origin_ty {
        Type::Path(tp) => {
            let wrapper = tp.path.segments.iter().last().unwrap();
            assert_eq!(wrapper.ident, "Vec", "expected Vec, but got {:?}", origin_ty);
            let generic_arg = match wrapper.arguments {
                PathArguments::AngleBracketed(ref params) => params.args.iter().next().unwrap(),
                _ => panic!("unexpected token {:?}", origin_ty),
            };
            match generic_arg {
                GenericArgument::Type(ty) => ty,
                _ => panic!("expected generic, but get {:?}", origin_ty),
            }
        },
        _ => panic!("unexpected type {:?}", origin_ty),
    };
    let set = Ident::new(&format!("set_{}", ident_str), Span::call_site());
    let get = Ident::new(&format!("get_{}", ident_str), Span::call_site());
    let take = Ident::new(&format!("take_{}", ident_str), Span::call_site());
    let mutation = Ident::new(&format!("mut_{}", ident_str), Span::call_site());
    quote! {
        pub fn #set(&mut self, value: #origin_ty) {
            self.#ident = value;
        }

        pub fn #get(&self) -> &[#ty] {
            &self.#ident
        }

        pub fn #mutation(&mut self) -> &mut #origin_ty {
            &mut self.#ident
        }

        pub fn #take(&mut self) -> #origin_ty {
            ::std::mem::replace(&mut self.#ident, ::std::vec::Vec::new())
        }
    }
}

fn classicalize_copy_field(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let mut ident_str = ident.to_string();
    if ident_str.starts_with("r#") {
        ident_str = ident_str[2..].to_owned();
    }
    let ty = &field.ty;
    let set = Ident::new(&format!("set_{}", ident_str), Span::call_site());
    let get = Ident::new(&format!("get_{}", ident_str), Span::call_site());
    quote! {
        pub fn #set(&mut self, value: #ty) {
            self.#ident = value;
        }

        pub fn #get(&self) -> #ty {
            self.#ident
        }
    }
}

fn classicalize_string_field(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let mut ident_str = ident.to_string();
    if ident_str.starts_with("r#") {
        ident_str = ident_str[2..].to_owned();
    }
    let set = Ident::new(&format!("set_{}", ident_str), Span::call_site());
    let get = Ident::new(&format!("get_{}", ident_str), Span::call_site());
    let take = Ident::new(&format!("take_{}", ident_str), Span::call_site());
    quote! {
        pub fn #set(&mut self, value: impl Into<String>) {
            self.#ident = value.into();
        }

        pub fn #get(&self) -> &str {
            &self.#ident
        }

        pub fn #take(&mut self) -> String {
            ::std::mem::replace(&mut self.#ident, String::new())
        }
    }
}

fn classicalize_bytes_field(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let mut ident_str = ident.to_string();
    if ident_str.starts_with("r#") {
        ident_str = ident_str[2..].to_owned();
    }
    let set = Ident::new(&format!("set_{}", ident_str), Span::call_site());
    let get = Ident::new(&format!("get_{}", ident_str), Span::call_site());
    let take = Ident::new(&format!("take_{}", ident_str), Span::call_site());
    quote! {
        pub fn #set(&mut self, value: impl Into<Vec<u8>>) {
            self.#ident = value.into();
        }

        pub fn #get(&self) -> &[u8] {
            &self.#ident
        }

        pub fn #take(&mut self) -> Vec<u8> {
            ::std::mem::replace(&mut self.#ident, Vec::new())
        }
    }
}


fn classicalize_enum_field(field: &Field, lit: &Lit) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let get = Ident::new(&format!("get_{}", ident), Span::call_site());
    let ty = match lit {
        Lit::Str(s) => syn::parse_str::<Path>(&s.value()).unwrap(),
        _ => panic!("expected enum type, but got {:?}", lit),
    };
    quote! {
        pub fn #get(&self) -> #ty {
            match #ty::from_i32(self.#ident) {
                Some(v) => v,
                None => panic!("Unexpected enum value for #lit: {}", self.#ident),
            }
        }
    }
}

#[derive(PartialEq)]
enum FieldType {
    Message,
    Copyable,
    String,
    Bytes,
}

#[derive(PartialEq)]
enum Frequency {
    Optional,
    Repeated,
}

fn classicalize_accessors(field: &Field) -> Option<proc_macro2::TokenStream> {
    for a in &field.attrs {
        for m in a.interpret_meta() {
            match m {
                Meta::List(MetaList { ident, nested, .. }) => {
                    if ident == "prost" {
                        let mut ft = FieldType::Copyable;
                        let mut freq = Frequency::Optional;
                        for n in nested {
                            match n {
                                NestedMeta::Meta(Meta::Word(w)) => if w == "message" {
                                    ft = FieldType::Message;
                                } else if w == "optional" {
                                    freq = Frequency::Optional;
                                } else if w == "repeated" {
                                    freq = Frequency::Repeated;
                                } else if w == "bytes" {
                                    ft = FieldType::Bytes;
                                } else if w == "string" {
                                    ft = FieldType::String;
                                }
                                NestedMeta::Meta(Meta::NameValue(nv)) => if nv.ident == "enumeration" {
                                    return Some(classicalize_enum_field(field, &nv.lit))
                                }
                                _ => ()
                            }
                        }
                        return Some(match freq {
                            Frequency::Repeated => classicalize_repeated_field(field),
                            Frequency::Optional => {
                                match ft {
                                    FieldType::Message => classicalize_optional_message_field(field),
                                    FieldType::Copyable => classicalize_copy_field(field),
                                    FieldType::String => classicalize_string_field(field),
                                    FieldType::Bytes => classicalize_bytes_field(field),
                                }
                            }
                        })
                    }
                },
                _ => (),
            }
        }
    }
    None
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
