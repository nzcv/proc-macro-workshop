mod utils;
use proc_macro::TokenStream;
use std::collections::HashMap;
use quote::quote;
use syn::{DeriveInput, GenericParam, parse_quote};
use syn::visit::{self, Visit};

struct TypePathVisitor {
    generic_type_names: Vec<String>,                        // 这个是筛选条件，里面记录了所有的泛型参数的名字，例如`T`,`U`等
    associated_types: HashMap<String, Vec<syn::TypePath>>,  // 这里记录了所有满足条件的语法树节点
}

impl<'ast> Visit<'ast> for TypePathVisitor {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.path.segments.len() >= 2 {
            let generic_type_name = node.path.segments[0].ident.to_string();
            if self.generic_type_names.contains(&generic_type_name) {
                self.associated_types.entry(generic_type_name).or_insert(Vec::new()).push(node.clone());
            }
        }
        visit::visit_type_path(self, node);
    }
}

fn get_generic_associated_types(st: &syn::DeriveInput) -> HashMap<String, Vec<syn::TypePath>> {

    let origin_generic_param_names: Vec<String> = st.generics.params.iter().filter_map(|f| {
        if let syn::GenericParam::Type(ty) = f {
            return Some(ty.ident.to_string())
        }
        return None
    }).collect();


    let mut visitor = TypePathVisitor {
        generic_type_names: origin_generic_param_names,
        associated_types: HashMap::new(),
    };


    visitor.visit_derive_input(st);
    return visitor.associated_types;
}


#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    do_derive(ast).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn do_derive(ast:DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &ast.ident;

    let all_associated_types  = get_generic_associated_types(&ast);


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

    let mut top_types = Vec::new();
    let mut ignore_types = Vec::new();

    for field in fields {
        if let Some(g) = utils::get_field_type_name(field)? {
            top_types.push(g);
        };
        if let Some(sub_type) = utils::get_phantomdata_sub_type_name(field)? {
            //eprintln!("sub_type {:#?}", sub_type);
            ignore_types.push(sub_type);
        }
    }

    let mut generics_new = ast.generics.clone();

    let wc = generics_new.make_where_clause();
    for (ident_literal, associate_type) in all_associated_types {
        for tp in associate_type {
            wc.predicates.push(syn::parse_quote!(#tp:std::fmt::Debug))
        }
    }

    /*
    for g in generics_new.params.iter_mut() {
        if let GenericParam::Type(t) = g {
            //eprintln!("{:#?}", t);
            let ty_literal = t.ident.to_string();
            if ignore_types.contains(&ty_literal) && !top_types.contains(&ty_literal) {
                continue;
            } else {
                t.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }
    */


    // https://docs.rs/syn/1.0.93/syn/struct.Generics.html#method.split_for_impl
    let (impl_generics, ty_generics, where_clause) = generics_new.split_for_impl();
    //eprintln!("impl_generics {:#?} ty_generics {:#?} where_clause {:#?}", impl_generics, ty_generics, where_clause);
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