use rustc_ast::tokenstream::TokenStream;
use rustc_ast::{self as ast, AngleBracketedArg, AngleBracketedArgs, GenericArg, token};
use rustc_expand::base::{DummyResult, ExpandResult, ExtCtxt, MacEager, MacroExpanderResult};
use rustc_parse::exp;
use rustc_span::{ErrorGuaranteed, Span, sym};
use thin_vec::thin_vec;

use crate::diagnostics;

fn new_error_since(cx: &ExtCtxt<'_>, err_count_before: usize) -> Option<ErrorGuaranteed> {
    (cx.dcx().err_count() > err_count_before).then(|| cx.dcx().has_errors()).flatten()
}

/// Expands `write_place!($place = $value)` to:
/// `::core::ops::place::WritePlace::write_place(k#raw_handle($place), ($value))`.
///
/// Function call argument evaluation evaluates the place handle first and `$value` second.
pub(crate) fn expand_write_place<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> MacroExpanderResult<'cx> {
    let (place, value) = match parse_write_place(cx, sp, tts) {
        Ok(parsed) => parsed,
        Err(guar) => return ExpandResult::Ready(DummyResult::any(sp, guar)),
    };

    let def_sp = cx.with_def_site_ctxt(sp);
    let place_sp = cx.with_def_site_ctxt(place.span);
    let raw_handle = cx.expr(place_sp, ast::ExprKind::RawHandle(place));
    // Wrap `value` in a synthetic paren so it is visited as a regular expression rather than a
    // `Call` argument (`OptExpr`), preserving `stmt_expr_attributes`/`#[cfg]` gating and avoiding
    // "around function argument" unused-delimiter lints.
    let value = cx.expr_paren(cx.with_def_site_ctxt(value.span), value);
    let path = cx.std_path(def_sp, &[sym::ops, sym::place, sym::WritePlace, sym::write_place]);
    let expr = cx.expr_call_global(def_sp, path, thin_vec![raw_handle, value]);
    ExpandResult::Ready(MacEager::expr(expr))
}

fn parse_write_place(
    cx: &ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Result<(Box<ast::Expr>, Box<ast::Expr>), ErrorGuaranteed> {
    let mut parser = cx.new_parser_from_tts(tts);

    if parser.token == token::Eof {
        return Err(cx.dcx().emit_err(diagnostics::WritePlaceExpectedAssign { span: sp }));
    }
    if parser.token == token::Eq {
        return Err(cx.dcx().emit_err(diagnostics::PlaceMacroExpectedPlace {
            span: parser.token.span,
            macro_name: "write_place",
        }));
    }

    let err_count_before = cx.dcx().err_count();
    let expr = parser.parse_expr().map_err(|err| err.emit_err())?;
    if let ast::ExprKind::Err(guar) = expr.kind {
        return Err(guar);
    }
    if let Some(guar) = new_error_since(cx, err_count_before) {
        return Err(guar);
    }
    if parser.token != token::Eof {
        parser.unexpected().map_err(|err| err.emit_err())?;
    }

    match expr.kind {
        ast::ExprKind::Assign(place, value, _) => Ok((place, value)),
        _ => Err(cx.dcx().emit_err(diagnostics::WritePlaceExpectedAssign { span: sp })),
    }
}

/// Expands `borrow_place!(@$place)` and `borrow_place!(@<$type> $place)` to:
/// `::core::ops::place::BorrowPlace::<$type>::borrow_place(k#raw_handle($place))`
/// (with `$type = _` when omitted).
pub(crate) fn expand_borrow_place<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> MacroExpanderResult<'cx> {
    let def_sp = cx.with_def_site_ctxt(sp);
    let (ty, place) = match parse_borrow_place(cx, sp, def_sp, tts) {
        Ok(parsed) => parsed,
        Err(guar) => return ExpandResult::Ready(DummyResult::any(sp, guar)),
    };

    let place_sp = cx.with_def_site_ctxt(place.span);
    let raw_handle = cx.expr(place_sp, ast::ExprKind::RawHandle(place));

    let ty_span = ty.span;
    let mut path =
        cx.std_path(def_sp, &[sym::ops, sym::place, sym::BorrowPlace, sym::borrow_place]);
    // Attach `<$ty>` to the `BorrowPlace` segment (second-to-last).
    let trait_seg_idx = path.segments.len() - 2;
    path.segments[trait_seg_idx].args =
        Some(Box::new(ast::GenericArgs::AngleBracketed(AngleBracketedArgs {
            span: ty_span,
            args: thin_vec![AngleBracketedArg::Arg(GenericArg::Type(ty))],
        })));

    let expr = cx.expr_call_global(def_sp, path, thin_vec![raw_handle]);
    ExpandResult::Ready(MacEager::expr(expr))
}

fn parse_borrow_place(
    cx: &ExtCtxt<'_>,
    sp: Span,
    def_sp: Span,
    tts: TokenStream,
) -> Result<(Box<ast::Ty>, Box<ast::Expr>), ErrorGuaranteed> {
    let mut parser = cx.new_parser_from_tts(tts);

    if !parser.eat(exp!(At)) {
        let span = if parser.token == token::Eof { sp } else { parser.token.span };
        return Err(cx.dcx().emit_err(diagnostics::BorrowPlaceExpectedAt { span }));
    }

    let ty = if parser.eat_lt() {
        let ty_errs_before = cx.dcx().err_count();
        let ty = parser.parse_ty().map_err(|err| err.emit_err())?;
        if let ast::TyKind::Err(guar) = ty.kind {
            return Err(guar);
        }
        if let Some(guar) = new_error_since(cx, ty_errs_before) {
            return Err(guar);
        }
        parser.expect_gt().map_err(|err| err.emit_err())?;
        ty
    } else {
        cx.ty_infer(def_sp)
    };

    if parser.token == token::Eof {
        return Err(cx.dcx().emit_err(diagnostics::PlaceMacroExpectedPlace {
            span: sp,
            macro_name: "borrow_place",
        }));
    }

    let place_errs_before = cx.dcx().err_count();
    let place = parser.parse_expr().map_err(|err| err.emit_err())?;
    if let ast::ExprKind::Err(guar) = place.kind {
        return Err(guar);
    }
    if let Some(guar) = new_error_since(cx, place_errs_before) {
        return Err(guar);
    }
    if parser.token != token::Eof {
        parser.unexpected().map_err(|err| err.emit_err())?;
    }

    Ok((ty, place))
}
