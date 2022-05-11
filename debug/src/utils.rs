#![allow(dead_code)]

use std::process::id;
use syn::{Result};

pub fn derive_get_struct_fields(ast: &syn::DeriveInput) -> Option<&syn::punctuated::Punctuated<syn::Field, syn::Token![,]>>{
    if let syn::Data::Struct(
        syn::DataStruct{
            fields: syn::Fields::Named(
                syn::FieldsNamed{
                    ref named,
                    ..
                }
            ),
            ..
        }
    ) = ast.data {
       return Some(named)
    }
    None
}

pub fn is_field_optional(field: &syn::Field) -> bool{
    if let syn::Type::Path(
        syn::TypePath{
            path:syn::Path{
                ref segments,
                ..
            },
            ..
        }
    ) = field.ty{
        if let Some(
            syn::PathSegment{
                ident,
                ..
            }
        ) = segments.last() {  // we need to check the lat one, so xxx::Option() can work
            if ident == "Option" {
                return true
            }
        }
    }
    return false
}

pub fn is_field(field: &syn::Field, name: String) -> bool{
    if let syn::Type::Path(
        syn::TypePath{
            path:syn::Path{
                ref segments,
                ..
            },
            ..
        }
    ) = field.ty{
        if let Some(
            syn::PathSegment{
                ident,
                ..
            }
        ) = segments.last() {  // we need to check the lat one, so xxx::Option() can work
            eprintln!("{:#?}", ident);
            if ident == &name {
                return true
            }
        }
    }
    return false
}

pub fn extract_inner_type(field: &syn::Field, container_ident: String) -> Option<&syn::Type>{
    if let syn::Type::Path(
        syn::TypePath{
            path:syn::Path{
                ref segments,
                ..
            },
            ..
        }
    ) = field.ty{
        if let Some(
            syn::PathSegment{
                ident,
                arguments,
            }
        ) = segments.last() {  // we need to check the lat one, so xxx::Optional() can work
            if ident.to_string() == container_ident {
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments{
                        args,
                        ..
                    }
                ) = arguments {
                    if let syn::GenericArgument::Type(
                        ty
                    ) = args.first().unwrap() {
                        return Some(ty)
                    }
                }
            }
        }
    }
    return None
}
/*
attr NameValue(
    MetaNameValue {
        path: Path {
            leading_colon: None,
            segments: [
                PathSegment {
                    ident: Ident {
                        ident: "debug",
                        span: #0 bytes(1023..1028),
                    },
                    arguments: None,
                },
            ],
        },
        eq_token: Eq,
        lit: Str(
            LitStr {
                token: "0b{:08b}",
            },
        ),
    },
)
*/
pub fn get_attr_lit(field: &syn::Field, ident_literal: &str) -> Option<String> {
    if let Some(attr) = field.attrs.last() {
        if let Ok(ref meta) = attr.parse_meta() {
            if meta.path().is_ident(ident_literal) {
                // eprintln!("meta {:#?}", meta);
                if let syn::Meta::NameValue(syn::MetaNameValue{ path, lit:syn::Lit::Str(lit), .. }) = meta {
                    return Some(lit.value())
                }
            }
        }
    }
    None
}

pub fn get_attr_name(field: &syn::Field, ident_literal: &str, attar_literal: &str) -> Option<Result<String>> {
    if let Some(attr) = field.attrs.last() {
        if let Ok(ref meta) = attr.parse_meta() {
            eprintln!("attr {:#?}", meta);
            if meta.path().is_ident(ident_literal) {
                if let syn::Meta::List(syn::MetaList{ nested, .. }) = meta {
                    if let Some(syn::NestedMeta::Meta( syn::Meta::NameValue(syn::MetaNameValue{ path, lit:syn::Lit::Str(lit), .. }))) = nested.last() {
                        return if path.is_ident(attar_literal) {
                            Some(Ok(lit.value()))
                        } else {
                            let err_msg = format!(r#"expected `{}({} = "...")`"#, ident_literal, attar_literal);
                            Some(Err(syn::Error::new_spanned(meta, err_msg)))
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn get_field_type_name(field: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath{ path:syn::Path{ ref segments, .. }, .. }) = field.ty {
        if let Some(syn::PathSegment{ ident, arguments, }) = segments.last() {
            return Ok(Some(ident.to_string()))
        }
    }
    Ok(None)
}

pub fn get_phantomdata_sub_type_name(field: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath{path: syn::Path{ref segments, ..}, ..}) = field.ty {
        if let Some(syn::PathSegment{ref ident, ref arguments}) = segments.last() {
            if ident == "PhantomData" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments{args, ..}) = arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path( ref gp))) = args.first() {
                        if let Some(generic_ident) = gp.path.segments.first() {
                            return Ok(Some(generic_ident.ident.to_string()))
                        }
                    }
                }
            }
        }
    }
    return Ok(None)
}