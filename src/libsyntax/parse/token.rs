import util::interner;
import util::interner::interner;
import std::map::{hashmap, str_hash};
import std::serialization::{serializer,
                            deserializer,
                            serialize_uint,
                            deserialize_uint,
                            serialize_i64,
                            deserialize_i64,
                            serialize_u64,
                            deserialize_u64,
                            serialize_bool,
                            deserialize_bool};

#[auto_serialize]
type str_num = uint;

#[auto_serialize]
enum binop {
    PLUS,
    MINUS,
    STAR,
    SLASH,
    PERCENT,
    CARET,
    AND,
    OR,
    SHL,
    SHR,
}

#[auto_serialize]
enum token {
    /* Expression-operator symbols. */
    EQ,
    LT,
    LE,
    EQEQ,
    NE,
    GE,
    GT,
    ANDAND,
    OROR,
    NOT,
    TILDE,
    BINOP(binop),
    BINOPEQ(binop),

    /* Structural symbols */
    AT,
    DOT,
    ELLIPSIS,
    COMMA,
    SEMI,
    COLON,
    MOD_SEP,
    RARROW,
    LARROW,
    DARROW,
    FAT_ARROW,
    LPAREN,
    RPAREN,
    LBRACKET,
    RBRACKET,
    LBRACE,
    RBRACE,
    POUND,
    DOLLAR,

    /* Literals */
    LIT_INT(i64, ast::int_ty),
    LIT_UINT(u64, ast::uint_ty),
    LIT_INT_UNSUFFIXED(i64),
    LIT_FLOAT(str_num, ast::float_ty),
    LIT_STR(str_num),

    /* Name components */
    IDENT(str_num, bool),
    UNDERSCORE,

    /* For interpolation */
    ACTUALLY(whole_nt),

    DOC_COMMENT(str_num),
    EOF,
}

#[auto_serialize]
/// For interpolation during macro expansion.
enum whole_nt {
    w_item(@ast::item),
    w_block(ast::blk),
    w_stmt(@ast::stmt),
    w_pat( @ast::pat),
    w_expr(@ast::expr),
    w_ty(  @ast::ty),
    w_ident(str_num, bool),
    w_path(@ast::path),
    w_tt(  @ast::token_tree), //needs @ed to break a circularity
    w_mtcs(~[ast::matcher])
}

fn binop_to_str(o: binop) -> ~str {
    alt o {
      PLUS { ~"+" }
      MINUS { ~"-" }
      STAR { ~"*" }
      SLASH { ~"/" }
      PERCENT { ~"%" }
      CARET { ~"^" }
      AND { ~"&" }
      OR { ~"|" }
      SHL { ~"<<" }
      SHR { ~">>" }
    }
}

fn to_str(in: interner<@~str>, t: token) -> ~str {
    alt t {
      EQ { ~"=" }
      LT { ~"<" }
      LE { ~"<=" }
      EQEQ { ~"==" }
      NE { ~"!=" }
      GE { ~">=" }
      GT { ~">" }
      NOT { ~"!" }
      TILDE { ~"~" }
      OROR { ~"||" }
      ANDAND { ~"&&" }
      BINOP(op) { binop_to_str(op) }
      BINOPEQ(op) { binop_to_str(op) + ~"=" }

      /* Structural symbols */
      AT { ~"@" }
      DOT { ~"." }
      ELLIPSIS { ~"..." }
      COMMA { ~"," }
      SEMI { ~"" }
      COLON { ~":" }
      MOD_SEP { ~"::" }
      RARROW { ~"->" }
      LARROW { ~"<-" }
      DARROW { ~"<->" }
      FAT_ARROW { ~"=>" }
      LPAREN { ~"(" }
      RPAREN { ~")" }
      LBRACKET { ~"[" }
      RBRACKET { ~"]" }
      LBRACE { ~"{" }
      RBRACE { ~"}" }
      POUND { ~"#" }
      DOLLAR { ~"$" }

      /* Literals */
      LIT_INT(c, ast::ty_char) {
        ~"'" + char::escape_default(c as char) + ~"'"
      }
      LIT_INT(i, t) {
        int::to_str(i as int, 10u) + ast_util::int_ty_to_str(t)
      }
      LIT_UINT(u, t) {
        uint::to_str(u as uint, 10u) + ast_util::uint_ty_to_str(t)
      }
      LIT_INT_UNSUFFIXED(i) {
        int::to_str(i as int, 10u)
      }
      LIT_FLOAT(s, t) {
        *interner::get(in, s) +
            ast_util::float_ty_to_str(t)
      }
      LIT_STR(s) {
        ~"\""
            + str::escape_default(*interner::get(in, s))
            + ~"\""
      }

      /* Name components */
      IDENT(s, _) {
        *interner::get(in, s)
      }
      UNDERSCORE { ~"_" }

      /* Other */
      DOC_COMMENT(s) { *interner::get(in, s) }
      EOF { ~"<eof>" }
      ACTUALLY(w_nt) {
        ~"an interpolated " +
            alt w_nt {
              w_item(*) { ~"item" } w_block(*) { ~"block" }
              w_stmt(*) { ~"statement" } w_pat(*) { ~"pattern" }
              w_expr(*) { ~"expression" } w_ty(*) { ~"type" }
              w_ident(*) { ~"identifier" } w_path(*) { ~"path" }
              w_tt(*) { ~"tt" } w_mtcs(*) { ~"matcher sequence" }
            }
      }
    }
}

pure fn can_begin_expr(t: token) -> bool {
    alt t {
      LPAREN { true }
      LBRACE { true }
      LBRACKET { true }
      IDENT(_, _) { true }
      UNDERSCORE { true }
      TILDE { true }
      LIT_INT(_, _) { true }
      LIT_UINT(_, _) { true }
      LIT_INT_UNSUFFIXED(_) { true }
      LIT_FLOAT(_, _) { true }
      LIT_STR(_) { true }
      POUND { true }
      AT { true }
      NOT { true }
      BINOP(MINUS) { true }
      BINOP(STAR) { true }
      BINOP(AND) { true }
      BINOP(OR) { true } // in lambda syntax
      OROR { true } // in lambda syntax
      MOD_SEP { true }
      ACTUALLY(w_expr(*)) | ACTUALLY(w_ident(*)) | ACTUALLY(w_block(*))
      | ACTUALLY(w_path(*)) { true }
      _ { false }
    }
}

fn is_lit(t: token) -> bool {
    alt t {
      LIT_INT(_, _) { true }
      LIT_UINT(_, _) { true }
      LIT_INT_UNSUFFIXED(_) { true }
      LIT_FLOAT(_, _) { true }
      LIT_STR(_) { true }
      _ { false }
    }
}

pure fn is_ident(t: token) -> bool {
    alt t { IDENT(_, _) { true } _ { false } }
}

pure fn is_plain_ident(t: token) -> bool {
    alt t { IDENT(_, false) { true } _ { false } }
}

pure fn is_bar(t: token) -> bool {
    alt t { BINOP(OR) | OROR { true } _ { false } }
}

/**
 * All the valid words that have meaning in the Rust language.
 *
 * Rust keywords are either 'contextual' or 'restricted'. Contextual
 * keywords may be used as identifiers because their appearance in
 * the grammar is unambiguous. Restricted keywords may not appear
 * in positions that might otherwise contain _value identifiers_.
 */
fn keyword_table() -> hashmap<~str, ()> {
    let keywords = str_hash();
    for contextual_keyword_table().each_key |word| {
        keywords.insert(word, ());
    }
    for restricted_keyword_table().each_key |word| {
        keywords.insert(word, ());
    }
    keywords
}

/// Keywords that may be used as identifiers
fn contextual_keyword_table() -> hashmap<~str, ()> {
    let words = str_hash();
    let keys = ~[
        ~"as",
        ~"else",
        ~"move",
        ~"of",
        ~"priv", ~"pub",
        ~"self", ~"send", ~"static",
        ~"to",
        ~"use",
        ~"with"
    ];
    for keys.each |word| {
        words.insert(word, ());
    }
    words
}

/**
 * Keywords that may not appear in any position that might otherwise contain a
 * _value identifier_. Restricted keywords may still be used as other types of
 * identifiers.
 *
 * Reasons:
 *
 * * For some (most?), if used at the start of a line, they will cause the
 *   line to be interpreted as a specific kind of statement, which would be
 *   confusing.
 *
 * * `true` or `false` as identifiers would always be shadowed by
 *   the boolean constants
 */
fn restricted_keyword_table() -> hashmap<~str, ()> {
    let words = str_hash();
    let keys = ~[
        ~"alt", ~"again", ~"assert",
        ~"break",
        ~"check", ~"class", ~"const", ~"copy",
        ~"do", ~"drop",
        ~"else", ~"enum", ~"export", ~"extern",
        ~"fail", ~"false", ~"fn", ~"for",
        ~"if", ~"iface", ~"impl", ~"import",
        ~"let", ~"log", ~"loop",
        ~"mod", ~"mut",
        ~"new",
        ~"owned",
        ~"pure",
        ~"ret",
        ~"struct",
        ~"true", ~"trait", ~"type",
        ~"unchecked", ~"unsafe",
        ~"while"
    ];
    for keys.each |word| {
        words.insert(word, ());
    }
    words
}

// Local Variables:
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
