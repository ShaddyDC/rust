use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

/// Mutability for the root place reference (`&raw mut` vs `&raw const`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutability {
    Mut,
    Const,
}

/// Context for desugaring place expressions.
#[derive(Clone, Debug)]
pub struct DesugarContext {
    pub mutability: Mutability,
    pub type_bindings: Vec<(syn::Ident, syn::Type)>,
}

impl Default for DesugarContext {
    fn default() -> Self {
        Self { mutability: Mutability::Mut, type_bindings: Vec::new() }
    }
}

/// Check if the remaining stream contains a `=>` token at the top level.
fn has_fat_arrow(input: ParseStream) -> bool {
    let fork = input.fork();
    while !fork.is_empty() {
        if fork.peek(syn::Token![=>]) {
            return true;
        }
        if fork.parse::<proc_macro2::TokenTree>().is_err() {
            break;
        }
    }
    false
}

/// Input parser for `raw_handle!` macro.
///
/// Syntax options:
/// 1. `raw_handle!(place_expr)`
/// 2. `raw_handle!(mut place_expr)` or `raw_handle!(const place_expr)`
/// 3. `raw_handle!(var: Type => place_expr)`
/// 4. `raw_handle!(mut var: Type => place_expr)`
pub struct RawHandleInput {
    pub context: DesugarContext,
    pub expr: syn::Expr,
}

impl Parse for RawHandleInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mutability = if input.peek(syn::Token![mut]) {
            input.parse::<syn::Token![mut]>()?;
            Mutability::Mut
        } else if input.peek(syn::Token![const]) {
            input.parse::<syn::Token![const]>()?;
            Mutability::Const
        } else {
            Mutability::Mut
        };

        let mut type_bindings = Vec::new();
        // Check if there are type bindings: e.g. `x: Struct => expr`
        if has_fat_arrow(input) {
            loop {
                let var: syn::Ident = input.parse()?;
                input.parse::<syn::Token![:]>()?;
                let ty: syn::Type = input.parse()?;
                type_bindings.push((var, ty));
                if input.peek(syn::Token![=>]) {
                    input.parse::<syn::Token![=>]>()?;
                    break;
                }
                input.parse::<syn::Token![,]>()?;
                if input.peek(syn::Token![=>]) {
                    input.parse::<syn::Token![=>]>()?;
                    break;
                }
            }
        }

        let expr: syn::Expr = input.parse()?;
        Ok(Self { context: DesugarContext { mutability, type_bindings }, expr })
    }
}

/// Input parser for `type_of!` macro.
///
/// Syntax options:
/// 1. `type_of!(place_expr)`
/// 2. `type_of!(var: Type => place_expr)`
pub struct TypeOfInput {
    pub context: DesugarContext,
    pub expr: syn::Expr,
}

impl Parse for TypeOfInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut type_bindings = Vec::new();
        if has_fat_arrow(input) {
            loop {
                let var: syn::Ident = input.parse()?;
                input.parse::<syn::Token![:]>()?;
                let ty: syn::Type = input.parse()?;
                type_bindings.push((var, ty));
                if input.peek(syn::Token![=>]) {
                    input.parse::<syn::Token![=>]>()?;
                    break;
                }
                input.parse::<syn::Token![,]>()?;
                if input.peek(syn::Token![=>]) {
                    input.parse::<syn::Token![=>]>()?;
                    break;
                }
            }
        }

        let expr: syn::Expr = input.parse()?;
        Ok(Self { context: DesugarContext { mutability: Mutability::Mut, type_bindings }, expr })
    }
}

/// Desugars a place expression into its type representation according to the spec:
///
/// - `$path`: paths that refer to local variables and statics -> `typeof($path)`
/// - `*$place`: dereferencing another place -> `<type_of!($place) as PlaceProxy>::Target`
/// - `$place[$expr]`: indexing -> `<type_of!($place) as Indexable<typeof($expr)>>`
/// - `$place.$ident`: field access -> `<field_of!(type_of!($place), $ident) as Subplace>::Target`
/// - `($place)`: parenthesized place expressions -> `type_of!($place)`
/// - `$value`: arbitrary expression -> compile error ("fail on this for now")
pub fn desugar_type_of(expr: &syn::Expr, ctx: &DesugarContext) -> Result<TokenStream, syn::Error> {
    match expr {
        syn::Expr::Path(syn::ExprPath { path, .. }) => {
            if let Some(ident) = path.get_ident()
                && let Some((_, ty)) = ctx.type_bindings.iter().find(|(id, _)| id == ident)
            {
                return Ok(quote!(#ty));
            }
            Ok(quote!(typeof(#path)))
        }
        syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Deref(_), expr: inner, .. }) => {
            let inner_ty = desugar_type_of(inner, ctx)?;
            Ok(quote!(<#inner_ty as PlaceProxy>::Target))
        }
        syn::Expr::Index(syn::ExprIndex { expr: base, index, .. }) => {
            let base_ty = desugar_type_of(base, ctx)?;
            Ok(quote!(<#base_ty as Indexable<typeof(#index)>>))
        }
        syn::Expr::Field(syn::ExprField { base, member, .. }) => {
            let base_ty = desugar_type_of(base, ctx)?;
            Ok(quote!(<field_of!(#base_ty, #member) as Subplace>::Target))
        }
        syn::Expr::Paren(syn::ExprParen { expr: inner, .. })
        | syn::Expr::Group(syn::ExprGroup { expr: inner, .. }) => desugar_type_of(inner, ctx),
        _ => Err(syn::Error::new_spanned(
            expr,
            format!(
                "expression `{}` is not a supported place expression in `type_of!`; \
                value expressions stored in temporaries are not supported yet",
                expr.to_token_stream()
            ),
        )),
    }
}

/// Desugars a place expression into a place handle according to the spec:
/// - `$path`: `LocalHandle::new(&raw {const,mut} $path)`
/// - `*$place`: `DerefPlace::deref_place(raw_handle!($place))`
/// - `$place[$expr]`: `IndexPlace::index_place(raw_handle!($place), $expr)`
/// - `$place.$ident`:
///   `ProjectPlace::<field_of!(type_of!($place), $ident)>::project_place(...)`
/// - `($place)`: `raw_handle!($place)`
/// - `$value`: compile error ("fail on this for now")
pub fn desugar_raw_handle(
    expr: &syn::Expr,
    ctx: &DesugarContext,
) -> Result<TokenStream, syn::Error> {
    match expr {
        syn::Expr::Path(syn::ExprPath { path, .. }) => {
            let raw_kw = match ctx.mutability {
                Mutability::Mut => quote!(&raw mut),
                Mutability::Const => quote!(&raw const),
            };
            Ok(quote!(LocalHandle::new(#raw_kw #path)))
        }
        syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Deref(_), expr: inner, .. }) => {
            let inner_handle = desugar_raw_handle(inner, ctx)?;
            Ok(quote!(DerefPlace::deref_place(#inner_handle)))
        }
        syn::Expr::Index(syn::ExprIndex { expr: base, index, .. }) => {
            let inner_handle = desugar_raw_handle(base, ctx)?;
            Ok(quote!(IndexPlace::index_place(#inner_handle, #index)))
        }
        syn::Expr::Field(syn::ExprField { base, member, .. }) => {
            let inner_handle = desugar_raw_handle(base, ctx)?;
            let base_ty = desugar_type_of(base, ctx)?;
            Ok(quote!(
                ProjectPlace::<field_of!(#base_ty, #member)>::project_place(
                    #inner_handle,
                    <field_of!(#base_ty, #member)>::default()
                )
            ))
        }
        syn::Expr::Paren(syn::ExprParen { expr: inner, .. })
        | syn::Expr::Group(syn::ExprGroup { expr: inner, .. }) => desugar_raw_handle(inner, ctx),
        _ => Err(syn::Error::new_spanned(
            expr,
            format!(
                "expression `{}` is not a supported place expression in `raw_handle!`; \
                value expressions stored in temporaries are not supported yet",
                expr.to_token_stream()
            ),
        )),
    }
}
