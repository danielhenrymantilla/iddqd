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

mod mangling;

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

fn equivalent_inner(input: TokenStream2) -> Result<TokenStream2> {
    handle_derive(WhichDerive::Equivalent, input)
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
    handle_derive(WhichDerive::Comparable, input)
}

enum WhichDerive {
    Equivalent,
    Comparable,
}

#[cfg(false)]
const REMINDER: &str = stringify! {
    pub trait Equivalent<K: ?Sized> {
        /// Compare self to `key` and return `true` if they are equal.
        fn equivalent(&self, key: &K) -> bool;
    }

    pub trait Comparable<K: ?Sized>: Equivalent<K> {
        /// Compare self to `key` and return their ordering.
        fn compare(&self, key: &K) -> Ordering;
    }
};

fn handle_derive(
    which_derive: WhichDerive,
    input: TokenStream2,
) -> Result<TokenStream2> {
    #[allow(clippy::type_complexity)] // Clippy skill issue.
    let (Trait @ _, method, ret, fold_field_outputs): (
        &TokenStream2,
        &TokenStream2,
        &TokenStream2,
        &mut dyn FnMut(&mut dyn Iterator<Item = _>) -> TokenStream2,
    ) = match which_derive {
        WhichDerive::Equivalent => (
            &quote! {
                ::iddqd::Equivalent
            },
            &quote! {
                equivalent
            },
            &quote! {
                -> ::core::primitive::bool
            },
            &mut |each_method: &mut dyn Iterator<Item = TokenStream2>| {
                quote! {
                    true #( && #each_method )*
                }
            },
        ),
        WhichDerive::Comparable => (
            &quote! {
                ::iddqd::Comparable
            },
            &quote! {
                compare
            },
            &quote! {
                -> ::core::cmp::Ordering
            },
            &mut |each_method: &mut dyn Iterator<Item = TokenStream2>| {
                quote! {
                    ::core::cmp::Ordering::Equal #(
                        .then_with(|| #each_method) )*
                }
            },
        ),
    };
    let input @ DeriveInput { ident: Type @ _, generics, .. }: &DeriveInput =
        &parse2(input)?;

    let (generics_mangled, mangling_visitor) =
        &mut mangling::mangled_generics(generics);

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
            .flat_map(|wc| wc.predicates.iter().cloned()),
    );

    /// Do we go for naïve non-perfect derives here?
    ///
    /// To illustrate, consider:
    ///
    /// ```rs
    /// #[derive(Equivalent)]
    /// struct Example<T>(*const T);
    /// ```
    ///
    ///   - Do we bound on the generics? (This is what the stdlib does.) For instance:
    ///
    ///     ```rs
    ///     impl<T, U> Equivalent<Example<U>> for Example<T>
    ///     where
    ///         T: Equivalent<U>, // 👈
    ///     {}
    ///     ```
    ///
    ///     This *usually* suffices, but can result in unnecessarily-bounded generics.
    ///
    ///     For instance, `Example<WeirdNonComparable>` won't compile, even though `*const …` can be
    ///     compared with any other pointer, no matter the pointee.
    ///
    ///   - Or do we bound based on the fields themselves? For instance:
    ///
    ///     ```rs
    ///     impl<TU> Equivalent<Example<U>> for Example<T>
    ///     where
    ///         *const T: Equivalent<*const U>, // 👈
    ///     {}
    ///     ```
    ///
    ///     This is as good as it gets (perfectly accurate impl), but has two drawbacks:
    ///
    ///       - it may fail on some "pathological"/odd/niche recursive data structures;
    ///
    ///       - as with most "over-adjusted/overfitting" stuff, it's a dangerous SemVer
    ///         future-proofing footgun: it leaks private field implementation details in the impl,
    ///         and adjustments to these may result in adjustments to the range/coverage of these
    ///         impls. If coverage is lost for any choice of types, then this is a breaking change.
    const PERFECT_DERIVES: bool = {
        // Let's go with "perfect" derives
        true
    };

    if !PERFECT_DERIVES {
        generics_both.make_where_clause().predicates.extend(
            iter::zip(generics.type_params(), generics_mangled.type_params())
                .map(|(l, r)| (&l.ident, &r.ident))
                .map(|(T @ _, U @ _)| -> WherePredicate {
                    parse_quote_spanned!(T.span()=>
                        #T : #Trait < #U >
                    )
                }),
        );
    }

    match &input.data {
        Data::Union(data_union) => bail!(
            "`union`s are not supported" => data_union.union_token,
        ),
        Data::Struct(data_struct) => {
            if PERFECT_DERIVES {
                generics_both.make_where_clause().predicates.extend(
                    data_struct.fields.iter().map(
                        |Field { ty, .. }| -> WherePredicate {
                            let mut mangled_ty: Type = ty.clone();
                            mangling_visitor.visit_type_mut(&mut mangled_ty);
                            parse_quote_spanned!(ty.span()=>
                                #ty : #Trait < #mangled_ty >
                            )
                        },
                    ),
                );
            }

            let (intro_both, _, where_clauses) = generics_both.split_for_impl();
            let fwd_lhs = generics.split_for_impl().1;
            let fwd_rhs = generics_mangled.split_for_impl().1;
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
                    #Trait< #Type #fwd_rhs >
                for
                    #Type #fwd_lhs
                #where_clauses
                {
                    fn #method(&self, key: & #Type #fwd_rhs) #ret {
                        #fn_body
                    }
                }
            ))
        }
        Data::Enum(data_enum) => {
            // Note: perfect derives for enums are way less problematic as the fields are all `pub`
            // anyways.
            if PERFECT_DERIVES {
                generics_both.make_where_clause().predicates.extend(
                    data_enum.variants.iter().flat_map(|v| {
                        v.fields
                            .iter()
                            .map(|Field { ty, .. }| -> WherePredicate {
                                let mut mangled_ty: Type = ty.clone();
                                mangling_visitor
                                    .visit_type_mut(&mut mangled_ty);
                                parse_quote_spanned!(ty.span()=>
                                    #ty : #Trait < #mangled_ty >
                                )
                            })
                            .collect::<Vec<_>>()
                    }),
                );
            }

            let (intro_both, _, where_clauses) = generics_both.split_for_impl();
            let fwd_lhs = generics.split_for_impl().1;
            let fwd_rhs = generics_mangled.split_for_impl().1;
            fn field_names_of_variant(
                v: &Variant,
            ) -> impl Iterator<Item = (Ident, Ident)> {
                v.fields.members().map(|each_field_name: Member| {
                    match each_field_name {
                        Member::Named(ident) => (
                            format_ident!("lhs_{ident}"),
                            format_ident!("rhs_{ident}"),
                        ),
                        Member::Unnamed(Index { index, span }) => (
                            format_ident!("lhs_{index}", span = span),
                            format_ident!("rhs_{index}", span = span),
                        ),
                    }
                })
            }
            fn each_method(
                Trait @ _: &TokenStream2,
                method: &TokenStream2,
                v: &Variant,
            ) -> impl Iterator<Item = TokenStream2> {
                field_names_of_variant(v).map(move |(lhs_field, rhs_field)| {
                    quote! {
                        #Trait::#method(
                            #lhs_field,
                            #rhs_field,
                        )
                    }
                })
            }
            let kleene = &quote!();
            let if_equivalent =
                if matches!(which_derive, WhichDerive::Equivalent) {
                    &[kleene][..]
                } else {
                    &[]
                };
            let enum_arms = (0..).zip(&data_enum.variants).map(|(i, v)| {
                let (each_field_name_lhs, each_field_name_rhs) =
                    field_names_of_variant(v).collect::<(Vec<_>, Vec<_>)>();
                let each_field_name = v.fields.members();
                let each_field_name_clone = each_field_name.clone();
                let fn_body =
                    fold_field_outputs(&mut each_method(Trait, method, v));
                let VariantName @ _ = &v.ident;
                let same_variant_match_arm = quote! {
                    (
                        #Type::#VariantName {
                            #(
                                #each_field_name: #each_field_name_lhs,
                            )*
                        },
                        #Type::#VariantName {
                            #(
                                #each_field_name_clone: #each_field_name_rhs,
                            )*
                        },
                    ) => #fn_body,
                };
                // if !comparable, an empty `TokenStream`, otherwise, match arms covering all the
                // non-matching cases (`for i in 0..num_variants: for k in 0..i: (k, i), (i, k)`).
                // (We could halve it by using a fallback over all the, say, `(i, k)` cases, but
                // that won't change the `O(n²)` complexity anyways, and it would prevent getting
                // exhasutive-matching-checking in our favor.)
                let if_comparable_fallback = matches!(
                    which_derive,
                    WhichDerive::Comparable
                )
                .then(|| {
                    // We don't have access to an `Ord`-comparable `mem::discriminant`, so we have
                    // to roll up our sleeves and DIO, by taking advantage of the following:
                    // `for{k < i}, variants[k] < variants[i]`, i.e.,
                    // `for variant in &variants[..i] { variant < variants[i] }`, i.e.,
                    // `#EachSubVariant < #VariantName`
                    let EachSubVariant @ _ =
                        data_enum.variants.iter().take(i).map(|v| &v.ident);
                    quote!(
                        #(
                            (
                                #Type::#EachSubVariant { .. },
                                #Type::#VariantName { .. },
                            ) => ::core::cmp::Ordering::Less,
                            (
                                #Type::#VariantName { .. },
                                #Type::#EachSubVariant { .. },
                            ) => ::core::cmp::Ordering::Greater,
                        )*
                    )
                });
                quote!(
                    #same_variant_match_arm

                    #if_comparable_fallback
                )
            });
            Ok(quote!(
                impl #intro_both
                    #Trait< #Type #fwd_rhs >
                for
                    #Type #fwd_lhs
                #where_clauses
                {
                    fn #method(&self, key: & #Type #fwd_rhs) #ret {
                        match (self, key) {
                            #(#enum_arms)*

                            #(#if_equivalent
                                _ => false,
                            )*
                        }
                    }
                }
            ))
        }
    }
}
