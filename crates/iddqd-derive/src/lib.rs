#![allow(nonstandard_style, clippy::style, unused_imports)]

use ::core::{iter, mem};
use ::proc_macro::TokenStream;
use ::proc_macro2::{Span, TokenStream as TokenStream2, TokenTree as TT};
use ::quote::{ToTokens, format_ident, quote, quote_spanned};
use ::syn::{
    Result,
    parse::{Parse, ParseBuffer, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::VisitMut,
    *,
};

#[rustfmt::skip]
macro_rules! bail {(
    $error:expr $(,)? => $spanned:expr $(,)?
) => (
    return Err(Error::new_spanned(&$spanned, $error))
)}

#[proc_macro_derive(Equivalent)]
pub fn equivalent(input: TokenStream) -> TokenStream {
    equivalent_inner(input.into())
        .map_err(|err| {
            let mut errors = err.into_iter().map(|err| {
                Error::new_spanned(
                    &err.to_compile_error(),
                    format!("`#[derive(Equivalent)]`: {err}"),
                )
            });
            let mut err = errors.next().unwrap();
            err.extend(errors);
            err
        })
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

struct Mangler {
    lifetimes: Vec<String>,
    type_params: Vec<String>,
}

fn generics_mangler(g: &Generics) -> (Mangler, Generics) {
    fn mangle_ident(ident: &Ident) -> Ident {
        format_ident!("__{ident}2")
    }

    use ::syn::visit_mut as subrecursing;

    #[expect(non_local_definitions)]
    impl VisitMut for Mangler {
        fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
            if self.lifetimes.contains(&lt.ident.to_string()) {
                lt.ident = mangle_ident(&lt.ident);
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

    let mut mangler = Mangler {
        lifetimes: g
            .lifetimes()
            .map(|lt| lt.lifetime.ident.to_string())
            .collect(),
        type_params: g.type_params().map(|tp| tp.ident.to_string()).collect(),
    };
    let mut ret = g.clone();
    mangler.visit_generics_mut(&mut ret);
    (mangler, ret)
}

const _: &str = stringify! {
    pub trait Equivalent<K: ?Sized> {
        /// Compare self to `key` and return `true` if they are equal.
        fn equivalent(&self, key: &K) -> bool;
    }

    pub trait Comparable<K: ?Sized>: Equivalent<K> {
        /// Compare self to `key` and return their ordering.
        fn compare(&self, key: &K) -> Ordering;
    }
};

fn equivalent_inner(input: TokenStream2) -> Result<TokenStream2> {
    common(
        input,
        &quote! {
            ::iddqd::Equivalent
        },
        &format_ident! {
            "equivalent"
        },
        &quote! {
            -> ::core::primitive::bool
        },
        |each_method| {
            quote!(
                true #(&&
                    #each_method
                )*
            )
        },
    )
}

#[proc_macro_derive(Comparable)]
pub fn comparable(input: TokenStream) -> TokenStream {
    comparable_inner(input.into())
        .map_err(|err| {
            let mut errors = err.into_iter().map(|err| {
                Error::new_spanned(
                    &err.to_compile_error(),
                    format!("`#[derive(Comparable)]`: {err}"),
                )
            });
            let mut err = errors.next().unwrap();
            err.extend(errors);
            err
        })
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn comparable_inner(input: TokenStream2) -> Result<TokenStream2> {
    common(
        input,
        &quote! {
            ::iddqd::Comparable
        },
        &format_ident! {
            "compare"
        },
        &quote! {
            -> ::core::cmp::Ordering
        },
        |each_method| {
            quote!(
                ::core::cmp::Ordering::Equal #(
                    .then_with(|| #each_method) )*
            )
        },
    )
}

fn common(
    input: TokenStream2,
    Trait @ _: &TokenStream2,
    method @ _: &Ident,
    ret: &TokenStream2,
    mut fold_field_outputs: impl FnMut(
        &mut dyn Iterator<Item = TokenStream2>,
    ) -> TokenStream2,
) -> Result<TokenStream2> {
    let input @ DeriveInput { ident: Type @ _, generics, .. }: &DeriveInput =
        &parse2(input)?;

    let (mangler, generics_mangled) = &mut generics_mangler(generics);

    let mut generics_both: Generics = generics.clone();
    for lt in generics_mangled.lifetimes() {
        generics_both.params.insert(0, GenericParam::Lifetime(lt.clone()));
    }
    generics_both.params.extend(
        generics_mangled.type_params().cloned().map(GenericParam::Type),
    );

    generics_both.make_where_clause().predicates.extend(
        (&generics_mangled.where_clause)
            .into_iter()
            .flat_map(|wc| wc.predicates.clone()),
    );

    match &input.data {
        Data::Union(data_union) => bail!(
            "`union`s are not supported" => data_union.union_token,
        ),
        Data::Struct(data_struct) => {
            generics_both.make_where_clause().predicates.extend(
                Iterator::chain(
                    iter::zip(
                        generics.type_params(),
                        generics_mangled.type_params(),
                    )
                    .map(|(l, r)| (&l.ident, &r.ident))
                    .map(
                        |(T @ _, U @ _)| -> WherePredicate {
                            parse_quote_spanned!(T.span()=>
                                #T : #Trait < #U >
                            )
                        },
                    ),
                    data_struct.fields.iter().map(|f| -> WherePredicate {
                        let ty = &f.ty;
                        let mut ty2: Type = ty.clone();
                        mangler.visit_type_mut(&mut ty2);
                        parse_quote_spanned!(ty.span()=>
                            #ty : #Trait < #ty2 >
                        )
                    }),
                ),
            );

            let (intro_both, _, where_clauses) = generics_both.split_for_impl();
            let fwd_lhs = generics.split_for_impl().1;
            let fwd_rhs = generics_mangled.split_for_impl().1;
            let Key @ _ = &quote! {
                #Type #fwd_rhs
            };
            let each_method =
                &mut data_struct.fields.members().map(|each_field_name| {
                    quote! {
                        #Trait::#method(
                            &self.#each_field_name,
                            &key.#each_field_name,
                        )
                    }
                });
            let fn_body = fold_field_outputs(each_method);
            Ok(quote!(
                impl #intro_both
                    #Trait< #Key >
                for
                    #Type #fwd_lhs
                #where_clauses
                {
                    fn #method(
                        &self,
                        key: &#Key,
                    ) #ret
                    {
                        #fn_body
                    }
                }
            ))
        }
        Data::Enum(_data_enum) => {
            todo!("`enum` support for `#[derive(Equivalent / Comparable]`")
        }
    }
}
