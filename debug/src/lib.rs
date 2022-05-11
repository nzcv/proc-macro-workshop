mod utils;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam, parse_quote};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    do_derive(ast).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn do_derive(ast:DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let ident_literal = ident.to_string();

    let fields = utils::derive_get_struct_fields(&ast).unwrap();
    let default_fmt_body:Vec<_> = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_name_literal = field_name.as_ref().unwrap().to_string();
        if let Some(format_literal) = utils::get_attr_lit(field, "debug") {
            quote!(
                .field(#field_name_literal, &format_args!(#format_literal ,&self.#field_name))
            )
        } else {
            quote!(
                .field(#field_name_literal, &self.#field_name)
            )
        }
    }).collect();

    let mut generics_new = ast.generics.clone();
    for mut g in generics_new.params.iter_mut() {
        if let GenericParam::Type(t) = g {
            eprintln!("{:#?}", t);
            t.bounds.push(parse_quote!(std::fmt::Debug));
            // t.bounds.push(syn::parse_quote!(std::fmt:Debug));
        }
    }

    // https://docs.rs/syn/1.0.93/syn/struct.Generics.html#method.split_for_impl
    let (impl_generics, ty_generics, where_clause) = generics_new.split_for_impl();
    // eprintln!("{:#?} {:#?} {:#?}", impl_generics, ty_generics, where_clause);
    let impl_debug = quote! {
        impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#ident_literal)
                #(#default_fmt_body)*
                .finish()
            }
        }
    };

    let ret = quote!(
        #impl_debug
    );

    Ok(ret.into())
}