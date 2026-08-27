use proc_macro2::{Delimiter, TokenStream, TokenTree};

/// Pretty-print a desugared place handle or type token stream into a formatted
/// multi-line string with clear indentation showing every level of desugaring.
pub fn format_pretty(tokens: &TokenStream) -> String {
    let mut out = String::new();
    let mut indent = 0;
    let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
    format_trees(&trees, &mut out, &mut indent, true);
    out.trim().to_string()
}

/// Compact single-line string formatting.
#[allow(dead_code)]
pub fn format_compact(tokens: &TokenStream) -> String {
    let mut s = tokens.to_string();
    s = s.replace(":: ", "::");
    s = s.replace(" ::", "::");
    s = s.replace(" < ", "<");
    s = s.replace(" > ", ">");
    s = s.replace(" , ", ", ");
    s = s.replace("& raw ", "&raw ");
    s
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("    ");
    }
}

fn format_trees(
    trees: &[TokenTree],
    out: &mut String,
    indent: &mut usize,
    parent_is_multiline: bool,
) {
    let mut i = 0;
    while i < trees.len() {
        let tree = &trees[i];
        match tree {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::None => (' ', ' '),
                };

                let inner_trees: Vec<TokenTree> = group.stream().into_iter().collect();
                // A group is multiline if it contains nested groups (e.g. nested calls)
                let has_nested_groups =
                    inner_trees.iter().any(|t| matches!(t, TokenTree::Group(_)));
                let is_multiline = has_nested_groups && inner_trees.len() > 3;

                if open != ' ' {
                    out.push(open);
                }

                if is_multiline {
                    *indent += 1;
                    out.push('\n');
                    push_indent(out, *indent);
                    format_trees(&inner_trees, out, indent, true);
                    *indent -= 1;
                    out.push('\n');
                    push_indent(out, *indent);
                } else {
                    format_trees(&inner_trees, out, indent, false);
                }

                if close != ' ' {
                    out.push(close);
                }
            }
            TokenTree::Ident(ident) => {
                let s = ident.to_string();
                if !out.is_empty() {
                    let last = out.chars().last().unwrap();
                    if last.is_alphanumeric() || last == '_' || last == '>' || last == ')' {
                        out.push(' ');
                    }
                }
                out.push_str(&s);
            }
            TokenTree::Punct(punct) => {
                let ch = punct.as_char();
                if ch == ',' {
                    out.push(',');
                    if parent_is_multiline {
                        out.push('\n');
                        push_indent(out, *indent);
                    } else {
                        out.push(' ');
                    }
                } else if ch == ';' {
                    out.push(';');
                    if parent_is_multiline {
                        out.push('\n');
                        push_indent(out, *indent);
                    } else {
                        out.push(' ');
                    }
                } else if ch == '&' {
                    if !out.is_empty() && !out.ends_with([' ', '\n', '(']) {
                        out.push(' ');
                    }
                    out.push('&');
                } else {
                    out.push(ch);
                }
            }
            TokenTree::Literal(lit) => {
                if !out.is_empty() {
                    let last = out.chars().last().unwrap();
                    if last.is_alphanumeric() || last == '_' {
                        out.push(' ');
                    }
                }
                out.push_str(&lit.to_string());
            }
        }
        i += 1;
    }
}
