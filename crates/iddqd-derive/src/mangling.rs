use super::*;

use ::syn::visit_mut as subrecursing;

fn mangle_ident(ident: &Ident) -> Ident {
    format_ident!("__{ident}2")
}

pub(super) struct ManglingVisitor {
    lifetimes: Vec<String>,
    type_params: Vec<String>,
}

impl VisitMut for ManglingVisitor {
    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        if self.lifetimes.contains(&lt.ident.to_string()) {
            lt.ident = mangle_ident(&lt.ident);
        }
    }

    fn visit_type_param_mut(&mut self, tp: &mut syn::TypeParam) {
        subrecursing::visit_type_param_mut(self, tp);
        let ident = &tp.ident;
        if self.type_params.contains(&ident.to_string()) {
            tp.ident = mangle_ident(ident);
        }
    }

    fn visit_type_path_mut(&mut self, ty: &mut TypePath) {
        subrecursing::visit_type_path_mut(self, ty);
        match (&ty.qself, ty.path.get_ident()) {
            (None, Some(ident))
                if self.type_params.contains(&ident.to_string()) =>
            {
                let ident = mangle_ident(ident);
                ty.path = parse_quote! { #ident };
            }
            _ => {}
        }
    }
}

/// "Mangle" a set of [`Generics`].
///
/// That is:
///
///  1. Identify/extract the set of lifetime and type params introduced by these [`Generics`].
///
///  1. Visit every single part of these [`Generics`] to replace each occurrence of these with
///     ["mangled"][`mangle_ident()`] versions thereof.
///
///  1. Return the resulting [`Generics`], as well as the helper [`ManglingVisitor`] should further
///     mangling-visiting be needed by the caller.
pub(super) fn mangled_generics(g: &Generics) -> (Generics, ManglingVisitor) {
    let mut mangling_visitor = ManglingVisitor {
        lifetimes: g
            .lifetimes()
            .map(|lt| lt.lifetime.ident.to_string())
            .collect(),
        type_params: g.type_params().map(|tp| tp.ident.to_string()).collect(),
    };
    let mut ret = g.clone();
    mangling_visitor.visit_generics_mut(&mut ret);
    (ret, mangling_visitor)
}
