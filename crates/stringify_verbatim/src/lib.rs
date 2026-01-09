//! A proc-macro that stringifies tokens while preserving original whitespace.
//!
//! Unlike the built-in `stringify!` macro which normalizes whitespace to single spaces,
//! `stringify_verbatim!` uses span information to reconstruct the original formatting.
//!
//! # Example
//!
//! ```
//! use stringify_verbatim::stringify_verbatim;
//!
//! // Built-in stringify normalizes whitespace:
//! // stringify!(foo   bar) → "foo bar"
//!
//! // stringify_verbatim preserves it:
//! let s = stringify_verbatim!(foo   bar);
//! assert_eq!(s, "foo   bar");
//! ```
//!
//! # Limitations
//!
//! - Comments are not preserved (they're not part of the token stream)
//! - Trailing whitespace after the last token cannot be captured
//! - Accuracy depends on `proc_macro2`'s span-locations feature

use proc_macro::TokenStream;
use proc_macro2::{LineColumn, TokenTree};

/// Stringify tokens while preserving original whitespace.
///
/// Returns a `&'static str` containing the tokens with their original spacing.
///
/// # Example
///
/// ```
/// use stringify_verbatim::stringify_verbatim;
///
/// let code = stringify_verbatim!(
///     fn example() {
///         42
///     }
/// );
/// assert!(code.contains("\n")); // Newlines preserved
/// ```
#[proc_macro]
pub fn stringify_verbatim(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let result = reconstruct_with_whitespace(input2);

    // Return as a string literal
    let lit = proc_macro2::Literal::string(&result);
    proc_macro2::TokenStream::from(proc_macro2::TokenTree::Literal(lit)).into()
}

/// Reconstruct the token stream as a string, preserving whitespace from span info.
fn reconstruct_with_whitespace(tokens: proc_macro2::TokenStream) -> String {
    let tts: Vec<TokenTree> = tokens.into_iter().collect();

    if tts.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;
    let mut i = 0;

    while i < tts.len() {
        // Try to detect and convert doc attributes back to /// or //! syntax
        if let Some((doc_comment, consumed, end_pos)) = try_parse_doc_attribute(&tts[i..]) {
            // Add whitespace before the doc comment
            if let Some(prev) = prev_end {
                let start = tts[i].span().start();
                let ws = compute_whitespace(prev, start);
                result.push_str(&ws);
            }

            result.push_str(&doc_comment);
            prev_end = Some(end_pos);
            i += consumed;
            continue;
        }

        let tt = &tts[i];
        let span = tt.span();
        let start = span.start();
        let end = span.end();

        // Add whitespace between previous token and this one
        if let Some(prev) = prev_end {
            let ws = compute_whitespace(prev, start);
            result.push_str(&ws);
        }

        // Add the token's text
        result.push_str(&token_to_string(tt));

        prev_end = Some(end);
        i += 1;
    }

    result
}

/// Try to parse a doc attribute pattern and convert it back to /// or //! syntax.
/// Returns (doc_comment_string, tokens_consumed, end_position) if successful.
fn try_parse_doc_attribute(tokens: &[TokenTree]) -> Option<(String, usize, LineColumn)> {
    // Pattern: # [ doc = "..." ] or # ! [ doc = "..." ]
    if tokens.is_empty() {
        return None;
    }

    // Check for #
    match &tokens[0] {
        TokenTree::Punct(p) if p.as_char() == '#' => {}
        _ => return None,
    }

    let mut idx = 1;
    let is_inner;

    // Check for optional !
    if idx < tokens.len() {
        if let TokenTree::Punct(p) = &tokens[idx] {
            if p.as_char() == '!' {
                is_inner = true;
                idx += 1;
            } else {
                is_inner = false;
            }
        } else {
            is_inner = false;
        }
    } else {
        return None;
    }

    // Check for [...]
    if idx >= tokens.len() {
        return None;
    }

    let group = match &tokens[idx] {
        TokenTree::Group(g) if g.delimiter() == proc_macro2::Delimiter::Bracket => g,
        _ => return None,
    };

    // Parse the group contents: doc = "..." or doc="..."
    let inner: Vec<TokenTree> = group.stream().into_iter().collect();

    // Need at least: doc = "string" (3 tokens) or doc="string" with no space (still 3)
    if inner.len() < 3 {
        return None;
    }

    // Check for "doc" ident
    let is_doc = match &inner[0] {
        TokenTree::Ident(i) => i.to_string() == "doc",
        _ => false,
    };

    if !is_doc {
        return None;
    }

    // Check for =
    let has_eq = match &inner[1] {
        TokenTree::Punct(p) => p.as_char() == '=',
        _ => false,
    };

    if !has_eq {
        return None;
    }

    // Check for string literal
    let doc_content = match &inner[2] {
        TokenTree::Literal(lit) => {
            let s = lit.to_string();
            // Remove quotes from the string literal
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                s[1..s.len() - 1].to_string()
            } else {
                return None;
            }
        }
        _ => return None,
    };

    // Successfully parsed a doc attribute!
    let prefix = if is_inner { "//!" } else { "///" };
    let doc_comment = format!("{}{}", prefix, doc_content);

    let end_pos = group.span().end();
    let consumed = idx + 1;

    Some((doc_comment, consumed, end_pos))
}

/// Compute the whitespace string between two positions.
fn compute_whitespace(from: LineColumn, to: LineColumn) -> String {
    if from.line == to.line {
        // Same line: just spaces
        let spaces = to.column.saturating_sub(from.column);
        " ".repeat(spaces)
    } else {
        // Different lines: newlines + indentation
        let newlines = to.line.saturating_sub(from.line);
        let mut ws = "\n".repeat(newlines);
        ws.push_str(&" ".repeat(to.column));
        ws
    }
}

/// Convert a token tree to its string representation.
fn token_to_string(tt: &TokenTree) -> String {
    match tt {
        TokenTree::Group(g) => {
            // For groups, we need to handle whitespace inside the delimiters
            let inner_tokens: Vec<TokenTree> = g.stream().into_iter().collect();
            let (open, close) = match g.delimiter() {
                proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                proc_macro2::Delimiter::Brace => ("{", "}"),
                proc_macro2::Delimiter::Bracket => ("[", "]"),
                proc_macro2::Delimiter::None => ("", ""),
            };

            if inner_tokens.is_empty() {
                return format!("{}{}", open, close);
            }

            // Get the group's span and first/last inner token spans
            let group_span = g.span();
            let first_token = inner_tokens.first().unwrap();
            let last_token = inner_tokens.last().unwrap();

            // Compute leading whitespace (between open delimiter and first token)
            // The group span start is approximately where the open delimiter is
            let group_start = group_span.start();
            let first_start = first_token.span().start();
            let leading_ws = if first_start.line == group_start.line {
                // Same line: compute column difference, minus 1 for the delimiter itself
                let diff = first_start.column.saturating_sub(group_start.column);
                if diff > 1 {
                    " ".repeat(diff - 1)
                } else {
                    String::new()
                }
            } else {
                // Different lines
                let newlines = first_start.line.saturating_sub(group_start.line);
                let mut ws = "\n".repeat(newlines);
                ws.push_str(&" ".repeat(first_start.column));
                ws
            };

            // Compute trailing whitespace (between last token and close delimiter)
            let group_end = group_span.end();
            let last_end = last_token.span().end();
            let trailing_ws = if last_end.line == group_end.line {
                let diff = group_end.column.saturating_sub(last_end.column);
                if diff > 1 {
                    " ".repeat(diff - 1)
                } else {
                    String::new()
                }
            } else {
                let newlines = group_end.line.saturating_sub(last_end.line);
                let mut ws = "\n".repeat(newlines);
                // Indentation for closing delimiter
                if group_end.column > 0 {
                    ws.push_str(&" ".repeat(group_end.column.saturating_sub(1)));
                }
                ws
            };

            // Reconstruct inner content
            let inner = reconstruct_with_whitespace(g.stream());

            format!("{}{}{}{}{}", open, leading_ws, inner, trailing_ws, close)
        }
        TokenTree::Ident(i) => i.to_string(),
        TokenTree::Punct(p) => p.to_string(),
        TokenTree::Literal(l) => l.to_string(),
    }
}

#[cfg(test)]
mod tests {
    // Tests need to be in a separate crate that uses this proc-macro
    // See tests/ directory
}
