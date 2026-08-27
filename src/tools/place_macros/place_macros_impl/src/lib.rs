use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

mod desugar;
mod format;
#[cfg(test)]
mod tests;

use desugar::{
    RawHandleInput, TypeOfInput, desugar_raw_handle as do_desugar_raw_handle,
    desugar_type_of as do_desugar_type_of,
};
use format::format_pretty;

/// Desugars a place expression into its corresponding `PlaceHandle`.
///
/// # Examples
///
/// ```ignore (illustrative syntax)
/// let hdl = raw_handle!(place);
/// let hdl = raw_handle!(mut place.field);
/// let hdl = raw_handle!(const place.field);
/// let hdl = raw_handle!(place: Struct => (*place.field[42])[8].other_field);
/// ```
#[proc_macro]
pub fn raw_handle(input: TokenStream) -> TokenStream {
    let RawHandleInput { context, expr } = parse_macro_input!(input as RawHandleInput);
    match do_desugar_raw_handle(&expr, &context) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Computes the type representation of a place expression.
///
/// # Examples
///
/// ```ignore (illustrative syntax)
/// type Ty = type_of!(place.field);
/// type Ty = type_of!(place: Struct => (*place.field).other);
/// ```
#[proc_macro]
pub fn type_of(input: TokenStream) -> TokenStream {
    let TypeOfInput { context, expr } = parse_macro_input!(input as TypeOfInput);
    match do_desugar_type_of(&expr, &context) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Returns a pretty-printed, indented string of the desugared `raw_handle!` code.
///
/// Useful for debugging, documentation, and generating code examples.
#[proc_macro]
pub fn desugar_raw_handle(input: TokenStream) -> TokenStream {
    let RawHandleInput { context, expr } = parse_macro_input!(input as RawHandleInput);
    match do_desugar_raw_handle(&expr, &context) {
        Ok(tokens) => {
            let formatted = format_pretty(&tokens);
            syn::LitStr::new(&formatted, proc_macro2::Span::call_site()).into_token_stream().into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

/// Returns a pretty-printed, indented string of the desugared `type_of!` expression.
///
/// Useful for debugging, documentation, and generating code examples.
#[proc_macro]
pub fn desugar_type_of(input: TokenStream) -> TokenStream {
    let TypeOfInput { context, expr } = parse_macro_input!(input as TypeOfInput);
    match do_desugar_type_of(&expr, &context) {
        Ok(tokens) => {
            let formatted = format_pretty(&tokens);
            syn::LitStr::new(&formatted, proc_macro2::Span::call_site()).into_token_stream().into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

/// Returns a comprehensive string showing both the handle desugaring and the type desugaring
/// for a given place expression.
#[proc_macro]
pub fn desugar_place(input: TokenStream) -> TokenStream {
    let RawHandleInput { context, expr } = parse_macro_input!(input as RawHandleInput);
    let handle_res = do_desugar_raw_handle(&expr, &context);
    let type_res = do_desugar_type_of(&expr, &context);

    match (handle_res, type_res) {
        (Ok(hdl_tokens), Ok(ty_tokens)) => {
            let expr_str = expr.to_token_stream().to_string();
            let hdl_pretty = format_pretty(&hdl_tokens);
            let ty_pretty = format_pretty(&ty_tokens);
            let full = format!(
                "Place Expression:\n    {}\n\nDesugared Handle:\n    {}\n\nDesugared Type:\n    {}",
                expr_str,
                hdl_pretty.replace('\n', "\n    "),
                ty_pretty.replace('\n', "\n    "),
            );
            syn::LitStr::new(&full, proc_macro2::Span::call_site()).into_token_stream().into()
        }
        (Err(err), _) | (_, Err(err)) => err.to_compile_error().into(),
    }
}
