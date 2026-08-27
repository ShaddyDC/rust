use quote::ToTokens;
use syn::parse_quote;

use crate::desugar::{DesugarContext, RawHandleInput, desugar_raw_handle, desugar_type_of};

#[test]
fn test_value_expression_fails() {
    let ctx = DesugarContext::default();
    let exprs: Vec<syn::Expr> = vec![
        parse_quote!(a + b),
        parse_quote!(42),
        parse_quote!("string_literal"),
        parse_quote!(foo()),
        parse_quote!(obj.method()),
        parse_quote!(|x| x + 1),
    ];

    for expr in exprs {
        let res = desugar_raw_handle(&expr, &ctx);
        assert!(res.is_err(), "Expected error for expr: {}", expr.to_token_stream());
        let err = res.unwrap_err();
        assert!(
            err.to_string().contains("not a supported place expression"),
            "Unexpected error message: {}",
            err
        );

        let res_ty = desugar_type_of(&expr, &ctx);
        assert!(res_ty.is_err(), "Expected error for expr in type_of: {}", expr.to_token_stream());
        assert!(res_ty.unwrap_err().to_string().contains("not a supported place expression"));
    }
}

#[test]
fn test_tuple_field_access() {
    let ctx = DesugarContext::default();
    let expr: syn::Expr = parse_quote!(pair.0);
    let res = desugar_raw_handle(&expr, &ctx).unwrap();
    let s = res.to_string();
    assert!(s.contains("0"));
    assert!(s.contains("ProjectPlace"));
}

#[test]
fn test_path_with_colons() {
    let input: RawHandleInput = syn::parse_str("my_mod::my_var").unwrap();
    assert!(input.context.type_bindings.is_empty());
    let res = desugar_raw_handle(&input.expr, &input.context).unwrap();
    assert!(res.to_string().contains("my_mod :: my_var"));
}

#[test]
fn test_path_with_colons_and_type_binding() {
    let input: RawHandleInput =
        syn::parse_str("x: my_mod::Struct => my_mod::my_var.field").unwrap();
    assert_eq!(input.context.type_bindings.len(), 1);
    let res = desugar_raw_handle(&input.expr, &input.context).unwrap();
    assert!(res.to_string().contains("ProjectPlace"));
}
