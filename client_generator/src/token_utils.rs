use syn::parse::{Error, Result};
use syn::spanned::Spanned;
use syn::{Expr, ExprPath, Ident, Lit, Member};

pub fn as_path(expr: &Expr) -> Result<&ExprPath> {
    if let Expr::Path(expr_path) = &expr {
        Ok(&expr_path)
    } else {
        Err(Error::new(expr.span(), "expected path"))
    }
}

pub fn as_str(expr: &Expr) -> Result<String> {
    if let Expr::Lit(ref expr_lit) = expr {
        if let Lit::Str(ref val) = &expr_lit.lit {
            Ok(val.value())
        } else {
            return Err(Error::new(expr.span(), "expected string literal"));
        }
    } else {
        return Err(Error::new(expr.span(), "expected string literal"));
    }
}

pub fn member_as_ident(expr: &Member) -> Result<&Ident> {
    if let Member::Named(ident) = &expr {
        Ok(&ident)
    } else {
        Err(Error::new(expr.span(), "expected named member"))
    }
}
