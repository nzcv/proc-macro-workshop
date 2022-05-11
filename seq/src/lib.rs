use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use syn::{braced, parse_macro_input, token, Field, Ident, Result, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Token;

#[derive(Debug)]
struct SeqParser {
    ident: syn::Ident,
    begin: i64,
    end: i64,
    body: proc_macro2::TokenStream
}

impl Parse for SeqParser {
    fn parse(input: ParseStream) ->syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let begin :syn::LitInt = input.parse()?;
        input.parse::<syn::Token![..]>()?;
        let end :syn::LitInt = input.parse()?;

        let body;
        syn::braced!(body in input);

        let body : proc_macro2::TokenStream = body.parse()?;

        Ok(
            SeqParser {
                ident,
                begin:  begin.base10_parse::<i64>()?,
                end: end.base10_parse::<i64>()?,
                body,
            }
        )
    }
}

impl SeqParser {
    fn expand(&self, ts: &proc_macro2::TokenStream, n: isize) -> proc_macro2::TokenStream {
        let ttree : Vec<_> = ts.clone().into_iter().collect();
        let mut ret = proc_macro2::TokenStream::new();
        let mut idx = 0;

        while idx < ttree.len() {
            let tree_node = &ttree[idx];
            match tree_node {
                TokenTree::Group(g) => {
                    let new_ts = self.expand(&g.stream(), n);
                    let wrap_in_group = proc_macro2::Group::new(g.delimiter(), new_ts);
                    ret.extend(quote::quote!(#wrap_in_group));
                }
                TokenTree::Ident(prefix) => {
                    if idx + 2 < ttree.len() {
                        if let proc_macro2::TokenTree::Punct(p) = &ttree[idx+1] {
                            if p.as_char() == '~' {
                                if let proc_macro2::TokenTree::Ident(ident) = &ttree[idx + 2] {
                                    if ident == &self.ident
                                        && prefix.span().end() == p.span().start()
                                        && p.span().end() == ident.span().start() {
                                        let new_ident_litral = format!("{}{}", prefix.to_string(), n);
                                        let new_ident = proc_macro2::Ident::new(new_ident_litral.as_str(), prefix.span());
                                        ret.extend(quote::quote!(#new_ident));
                                        idx += 3;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    if prefix == &self.ident {
                        let new_i = proc_macro2::Literal::i64_unsuffixed(n as i64);
                        ret.extend(quote::quote!(#new_i));
                    } else {
                        ret.extend(quote::quote!(#tree_node));
                    }
                }
                _ => {
                    ret.extend(quote::quote!(#tree_node))
                }
                //TokenTree::Punct(_) => {}
                //TokenTree::Literal(_) => {}
            }
            idx += 1
        }
        ret
    }

    fn find_block_to_expand_and_do_expand(&self, c: syn::buffer::Cursor) -> (proc_macro2::TokenStream, bool) {
        let mut found = false;
        let mut ret = proc_macro2::TokenStream::new();
        let mut cursor = c;
        while !cursor.eof() {
            if let Some((purct_prefix, cursor_1)) = cursor.punct() {
                if purct_prefix.as_char() == '#' {
                    if let Some((group_cur,_,cursor_2)) = cursor_1.group(proc_macro2::Delimiter::Parenthesis) {
                        if let Some((puct_suffix, cursor_3)) = cursor_2.punct() {
                            if puct_suffix.as_char() == '*' {
                                for i in self.begin..self.end {
                                    let t = self.expand(&group_cur.token_stream(), i as isize);
                                    ret.extend(t);
                                }

                                cursor = cursor_3;
                                found = true;
                                continue
                            }
                        }
                    }
                }
            }
        }
    }
}

// seq!(N in 0..8 {
//     // nothing
// });

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as SeqParser);
    let mut ret = proc_macro2::TokenStream::new();
    for i in st.begin..st.end {
        let ts = st.expand(&st.body, i as isize);
        ret.extend(ts);
    }
    ret.into()
}
