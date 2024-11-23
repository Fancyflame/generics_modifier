//! implementations are modified from repository [`syn`](https://github.com/dtolnay/syn).

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{AttrStyle, Attribute, ConstParam, Token, TypeParam};

use crate::{GenericsModifier, KnownParam, TypeConstParam};

pub struct ImplGenerics<'a>(pub(super) &'a GenericsModifier);

impl<'a> ToTokens for ImplGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let this = &self.0;

        if this.lifetimes.is_empty() && this.types.is_empty() {
            return;
        }

        default_token::<Token![<]>(tokens);

        for (param, known) in this.lifetimes.iter() {
            if known.is_some() {
                continue;
            }

            param.to_tokens(tokens);
            default_token::<Token![,]>(tokens);
        }

        for (param, known) in this.types.iter() {
            if known.is_some() {
                continue;
            }

            match param {
                TypeConstParam::Type(param) => {
                    // Leave off the type parameter defaults
                    tokens.append_all(attrs_outer(&param.attrs));
                    param.ident.to_tokens(tokens);
                    if !param.bounds.is_empty() {
                        default_token::<Token![:]>(tokens);
                        param.bounds.to_tokens(tokens);
                    }
                }
                TypeConstParam::Const(param) => {
                    // Leave off the const parameter defaults
                    tokens.append_all(attrs_outer(&param.attrs));
                    param.const_token.to_tokens(tokens);
                    param.ident.to_tokens(tokens);
                    param.colon_token.to_tokens(tokens);
                    param.ty.to_tokens(tokens);
                }
            }

            default_token::<Token![,]>(tokens);
        }

        default_token::<Token![>]>(tokens);
    }
}

pub struct TypeGenerics<'a>(pub(super) &'a GenericsModifier);

impl<'a> ToTokens for TypeGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let this = &self.0;

        if this.lifetimes.is_empty() && this.types.is_empty() {
            return;
        }

        default_token::<Token![<]>(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        for (param, known) in this.lifetimes.iter() {
            match known {
                Some(lifetime) => match lifetime {
                    KnownParam::Lifetime(l) => l.to_tokens(tokens),
                    _ => unreachable!(),
                },
                None => {
                    // Leave off the lifetime bounds and attributes
                    param.lifetime.to_tokens(tokens);
                }
            }
            default_token::<Token![,]>(tokens);
        }

        let mut use_default = false;
        for (param, known) in this.types.iter() {
            let type_to_put: &dyn ToTokens = match known {
                Some(param) => match param {
                    KnownParam::Const(expr) => expr,
                    KnownParam::Type(ty) => ty,
                    KnownParam::UseDefault => {
                        use_default = true;
                        continue;
                    }
                    _ => unreachable!(),
                },
                None => {
                    let (TypeConstParam::Const(ConstParam { ident, .. })
                    | TypeConstParam::Type(TypeParam { ident, .. })) = param;
                    ident
                }
            };

            if use_default {
                panic!(
                    "cannot using types (using `{}`) after using default type",
                    type_to_put.to_token_stream().to_string()
                );
            }

            type_to_put.to_tokens(tokens);
            default_token::<Token![,]>(tokens);
        }

        default_token::<Token![>]>(tokens);
    }
}

fn default_token<T>(tokens: &mut TokenStream)
where
    T: ToTokens + Default,
{
    T::default().to_tokens(tokens);
}

fn attrs_outer(attrs: &Vec<Attribute>) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| match attr.style {
        AttrStyle::Outer => true,
        AttrStyle::Inner(_) => false,
    })
}
