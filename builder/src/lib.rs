mod utils;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    do_derive(ast).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn do_derive(ast:DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &ast.ident;
    let builder_ident = quote::format_ident!("{}Builder", struct_name);

    let fields = utils::derive_get_struct_fields(&ast).unwrap();
    
    let gen_builder_fields : Vec<_> = fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;

        if utils::is_field(field, "Option".into()) {
            Ok(quote!(#ident: #ty))
        } else if let Some(attr_name) = utils::get_each_attr_name(field) {
            match attr_name {
                Ok(attr_name) => {
                    Ok(quote!(#ident: #ty))
                }
                Err(e) => {
                    Err(e)
                }
            }
        } else {
            Ok(quote!(#ident : Option<#ty>))
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    let gen_builder_default : Vec<_> = fields.iter().map(|field| {
        let ident = &field.ident;
        let field_type =  &field.ty;
        if utils::get_each_attr_name(field).is_some() {
            quote! {
                #ident : <#field_type>::new()
            }
        } else {
            quote!(
                #ident : None
            )
        }
    }).collect();

    let gen_setters: Vec<_> = fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        if utils::is_field_optional(field) {
            if let Some(inner_ty) = utils::extract_inner_type(field, "Option".into()) {
                Ok(quote!{
                    fn #ident(&mut self, #ident: #inner_ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                })
            } else {
                Ok(quote!())
            }
        } else if let Some(Ok(attr_name)) = utils::get_each_attr_name(field) {
            let inner_type = utils::extract_inner_type(field, "Vec".into());
            let attr_ident = quote::format_ident!("{}", attr_name);
            Ok(quote!(
                fn #attr_ident(&mut self, #attr_ident: #inner_type) -> &mut Self {
                    self.#ident.push(#attr_ident);
                    self
                }
            ))
        } else {
            Ok(quote!{
                fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            })
        }
    }).collect::<syn::Result<Vec<_>>>()?;
    
    let gen_build_check_err: Vec<_> = fields.iter().map(|field| {
        let field_name = &field.ident;
        let missing_msg = format!("Field {:?} is missing", field_name);
        if utils::is_field_optional(field) {
            quote!()
        } else if utils::get_each_attr_name(field).is_some() {
            quote!()
        } else {
            quote!{
                if let std::option::Option::None = self.#field_name {
                    return std::result::Result::Err(#missing_msg.into())
                }
            }
        }
    }).collect();

    let gen_build_body: Vec<_> = fields.iter().map(|field| {
        let ident = &field.ident;
        // let ty = &field.ty;
        if utils::is_field_optional(field) {
            quote!{
                #ident : self.#ident.clone()
            }
        } else if utils::get_each_attr_name(field).is_some() {
            quote!{
                #ident : self.#ident.clone()
            }
        } else {
            quote!{
                #ident : self.#ident.clone().unwrap()
            }
        }
    }).collect();

    let gen_build = quote!{
        fn build(&mut self) -> std::result::Result<#struct_name, std::boxed::Box<dyn std::error::Error>> {
            #(#gen_build_check_err)*
            std::result::Result::Ok(
                #struct_name{
                    #(#gen_build_body),*
                }
            )
        }
    };

    // eprintln!("{:#?}", fields);

    /*
    let gen_fn_each: Vec<_>= fields.iter().map(|field| {
        let ident = &field.ident;
        if let Some(Ok(attr_name)) = utils::get_each_attr_name(field) {
            let inner_type = utils::extract_inner_type(field, "Vec".into());
            let attr_ident = quote::format_ident!("{}", attr_name);

            quote!(
                fn #attr_ident(&mut self, #attr_ident: #inner_type) -> &mut Self {
                    self.#ident.push(#attr_ident);
                    self
                }
            )
        } else {
            quote!()
        }
    }).collect();
    */

    let derive = quote!{
        pub struct #builder_ident {
            #(#gen_builder_fields),*
        }

        impl #builder_ident {
            #(#gen_setters)*
            #gen_build
        }

        impl #struct_name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#gen_builder_default),*
                }
            }
        }
    };

    Ok(derive.into())
}
