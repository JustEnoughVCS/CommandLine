use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ItemFn, Lit, Type, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn exec(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_block = &input_fn.block;

    let mut output_mappings = Vec::new();
    extract_cmd_output_macros(fn_block, &mut output_mappings);

    let mapping_fn = generate_mapping_function(&output_mappings);

    let expanded = quote! {
        #input_fn

        #mapping_fn
    };

    TokenStream::from(expanded)
}

fn extract_cmd_output_macros(block: &syn::Block, mappings: &mut Vec<(String, syn::Type)>) {
    use syn::visit::Visit;

    struct CmdOutputVisitor<'a> {
        mappings: &'a mut Vec<(String, syn::Type)>,
    }

    impl<'a> syn::visit::Visit<'a> for CmdOutputVisitor<'a> {
        fn visit_macro(&mut self, mac: &'a syn::Macro) {
            if mac.path.is_ident("cmd_output") {
                let nested_result = syn::parse2::<syn::ExprTuple>(mac.tokens.clone());
                if let Ok(nested) = nested_result {
                    if nested.elems.len() < 2 {
                        syn::visit::visit_macro(self, mac);
                        return;
                    }

                    let first_elem = &nested.elems[0];
                    let second_elem = &nested.elems[1];

                    let type_path_opt = match first_elem {
                        Expr::Path(path) => Some(path),
                        _ => None,
                    };

                    let lit_str_opt = match second_elem {
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Str(lit_str) => Some(lit_str),
                            _ => None,
                        },
                        _ => None,
                    };

                    if let (Some(type_path), Some(lit_str)) = (type_path_opt, lit_str_opt) {
                        let type_name = lit_str.value();
                        if let Some(type_ident) = type_path.path.get_ident() {
                            let ty: Type = parse_quote!(#type_ident);
                            self.mappings.push((type_name, ty));
                        }
                    }
                }
            }

            syn::visit::visit_macro(self, mac);
        }
    }

    let mut visitor = CmdOutputVisitor { mappings };
    visitor.visit_block(block);
}

fn generate_mapping_function(mappings: &[(String, syn::Type)]) -> proc_macro2::TokenStream {
    let mapping_entries: Vec<_> = mappings
        .iter()
        .map(|(name, ty)| {
            quote! {
                map.insert(#name.to_string(), std::any::TypeId::of::<#ty>());
            }
        })
        .collect();

    quote! {
        fn get_output_type_mapping() -> std::collections::HashMap<String, std::any::TypeId> {
            let mut map = std::collections::HashMap::new();
            #(#mapping_entries)*
            map
        }
    }
}
