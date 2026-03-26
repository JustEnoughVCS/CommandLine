use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, LitStr, Token, parse_macro_input};

struct SuggestInput {
    items: Punctuated<SuggestItem, Token![,]>,
}

enum SuggestItem {
    WithDesc(Box<(LitStr, Expr)>), // "-i" = "Insert something"
    Simple(LitStr),                // "-I"
}

impl Parse for SuggestInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::parse_terminated(input)?;
        Ok(SuggestInput { items })
    }
}

impl Parse for SuggestItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: LitStr = input.parse()?;

        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            let value: Expr = input.parse()?;
            Ok(SuggestItem::WithDesc(Box::new((key, value))))
        } else {
            Ok(SuggestItem::Simple(key))
        }
    }
}

#[proc_macro]
pub fn suggest(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SuggestInput);

    let mut tokens = TokenStream2::new();

    tokens.extend(quote! {
        CompletionResult::empty_comp()
    });

    for item in input.items {
        match item {
            SuggestItem::WithDesc(boxed) => {
                let (key, value) = *boxed;
                tokens.extend(quote! {
                    .with_suggest_desc(#key, #value)
                });
            }
            SuggestItem::Simple(key) => {
                tokens.extend(quote! {
                    .with_suggest(#key)
                });
            }
        }
    }

    tokens.into()
}

#[proc_macro]
pub fn file_suggest(_input: TokenStream) -> TokenStream {
    quote! {
        CompletionResult::file_comp()
    }
    .into()
}
