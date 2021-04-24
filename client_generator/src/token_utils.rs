use syn::parse::{Error, Result};
use syn::spanned::Spanned;
use syn::{Expr, Lit};

pub fn get_identifier(expr: &Expr) -> Result<String> {
    if let Expr::Path(ref expr_path) = expr {
        if expr_path.path.leading_colon.is_some() {
            return Err(Error::new(expr_path.span(), "expected unscoped identifier"));
        }
        if expr_path.path.segments.len() != 1 {
            return Err(Error::new(expr_path.span(), "expected unscoped identifier"));
        }

        Ok(expr_path.path.segments.first().unwrap().ident.to_string())
    } else {
        return Err(Error::new(expr.span(), "expected service name"));
    }
}

pub fn get_str(expr: &Expr) -> Result<String> {
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
