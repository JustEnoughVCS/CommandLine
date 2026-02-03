use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Type, parse_macro_input, spanned::Spanned};

/// Macro for simplifying renderer definitions
///
/// Expands the `#[result_renderer(Renderer)]` macro into the corresponding struct and trait implementation
///
/// # Example
/// ```ignore
/// #[result_renderer(MyRenderer)]
/// async fn render(data: &Output) -> Result<JVRenderResult, CmdRenderError> {
///     // Rendering logic
/// }
/// ```
///
/// Expands to:
/// ```ignore
/// pub struct MyRenderer;
///
/// impl JVResultRenderer<Output> for MyRenderer {
///     async fn render(data: &Output) -> Result<JVRenderResult, CmdRenderError> {
///         // Rendering logic
///     }
/// }
///
/// impl JVResultAutoRenderer<Output> for MyRenderer {
///     fn get_type_id(&self) -> std::any::TypeId {
///         std::any::TypeId::of::<Self>()
///     }
///
///     fn get_data_type_id(&self) -> std::any::TypeId {
///         std::any::TypeId::of::<Output>()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn result_renderer(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse macro arguments (renderer struct name)
    let renderer_name = parse_macro_input!(args as syn::Ident);

    // Parse the input function
    let input_fn = parse_macro_input!(input as ItemFn);

    // Check if the function is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new(input_fn.sig.ident.span(), "renderer function must be async")
            .to_compile_error()
            .into();
    }

    // Get the function name
    let fn_name = &input_fn.sig.ident;

    // Get function parameters
    let fn_inputs = &input_fn.sig.inputs;

    // Check the number of function parameters
    if fn_inputs.len() != 1 {
        return syn::Error::new(
            input_fn.sig.paren_token.span.join(),
            "renderer function must have exactly one parameter",
        )
        .to_compile_error()
        .into();
    }

    // Extract the type of the first parameter
    let param_type = match &fn_inputs[0] {
        syn::FnArg::Typed(pat_type) => &pat_type.ty,
        syn::FnArg::Receiver(_) => {
            return syn::Error::new(
                fn_inputs[0].span(),
                "renderer function cannot have self parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    // Check if the parameter type is a reference type, and extract the inner type
    let inner_type = match &**param_type {
        Type::Reference(type_ref) => {
            // Ensure it's a reference type
            &type_ref.elem
        }
        _ => {
            return syn::Error::new(
                param_type.span(),
                "renderer function parameter must be a reference type (&Data)",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract the parameter pattern (for function calls)
    let param_pattern = match &fn_inputs[0] {
        syn::FnArg::Typed(pat_type) => &pat_type.pat,
        _ => unreachable!(),
    };

    // Extract the function's visibility modifier
    let visibility = &input_fn.vis;

    // Extract generic parameters (if any)
    let generics = &input_fn.sig.generics;

    // Extract where clause (if any)
    let where_clause = &generics.where_clause;

    // Build the output
    let expanded = quote! {
        #input_fn

        #visibility struct #renderer_name;

        impl #generics crate::systems::render::renderer::JVResultRenderer<#inner_type> for #renderer_name
        #where_clause
        {
            fn render(
                #fn_inputs
            ) -> impl ::std::future::Future<Output = ::std::result::Result<
                crate::systems::render::renderer::JVRenderResult,
                crate::systems::cmd::errors::CmdRenderError
            >> + ::std::marker::Send + ::std::marker::Sync {
                async move {
                    #fn_name(#param_pattern).await
                }
            }

            fn get_type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn get_data_type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<#inner_type>()
            }
        }
    };

    expanded.into()
}
