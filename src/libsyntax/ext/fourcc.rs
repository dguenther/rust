// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/* The compiler code necessary to support the fourcc! extension. */

// fourcc!() is called with a single 4-character string, and an optional ident
// that is either `big` or `little`. If the ident is omitted it is assumed to
// be the platform-native value. It returns a u32.

use ast;
use attr::contains;
use codemap::{Span, mk_sp};
use ext::base::*;
use ext::base;
use ext::build::AstBuilder;
use parse;
use parse::token;
use parse::token::InternedString;

use std::ascii::AsciiCast;

pub fn expand_syntax_ext(cx: &mut ExtCtxt, sp: Span, tts: &[ast::TokenTree]) -> base::MacResult {
    let (expr, endian) = parse_tts(cx, tts);

    let little = match endian {
        None => false,
        Some(Ident{ident, span}) => match token::get_ident(ident.name).get() {
            "little" => true,
            "big" => false,
            "target" => target_endian_little(cx, sp),
            _ => {
                cx.span_err(span, "invalid endian directive in fourcc!");
                target_endian_little(cx, sp)
            }
        }
    };

    let s = match expr.node {
        // expression is a literal
        ast::ExprLit(lit) => match lit.node {
            // string literal
            ast::LitStr(ref s, _) => {
                if !s.get().is_ascii() {
                    cx.span_err(expr.span, "non-ascii string literal in fourcc!");
                } else if s.get().len() != 4 {
                    cx.span_err(expr.span, "string literal with len != 4 in fourcc!");
                }
                s
            }
            _ => {
                cx.span_err(expr.span, "unsupported literal in fourcc!");
                return MRExpr(cx.expr_lit(sp, ast::LitUint(0u64, ast::TyU32)));
            }
        },
        _ => {
            cx.span_err(expr.span, "non-literal in fourcc!");
            return MRExpr(cx.expr_lit(sp, ast::LitUint(0u64, ast::TyU32)));
        }
    };

    let mut val = 0u32;
    if little {
        for byte in s.get().bytes_rev().take(4) {
            val = (val << 8) | (byte as u32);
        }
    } else {
        for byte in s.get().bytes().take(4) {
            val = (val << 8) | (byte as u32);
        }
    }
    let e = cx.expr_lit(sp, ast::LitUint(val as u64, ast::TyU32));
    MRExpr(e)
}

struct Ident {
    ident: ast::Ident,
    span: Span
}

fn parse_tts(cx: &ExtCtxt, tts: &[ast::TokenTree]) -> (@ast::Expr, Option<Ident>) {
    let p = &mut parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_owned());
    let ex = p.parse_expr();
    let id = if p.token == token::EOF {
        None
    } else {
        p.expect(&token::COMMA);
        let lo = p.span.lo;
        let ident = p.parse_ident();
        let hi = p.last_span.hi;
        Some(Ident{ident: ident, span: mk_sp(lo, hi)})
    };
    if p.token != token::EOF {
        p.unexpected();
    }
    (ex, id)
}

fn target_endian_little(cx: &ExtCtxt, sp: Span) -> bool {
    let meta = cx.meta_name_value(sp, InternedString::new("target_endian"),
        ast::LitStr(InternedString::new("little"), ast::CookedStr));
    contains(cx.cfg(), meta)
}
