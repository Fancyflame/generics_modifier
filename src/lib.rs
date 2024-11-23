use linked_table::LinkedTable;
use proc_macro2::TokenStream;
use quote::ToTokens;
use split_for_impl::{ImplGenerics, TypeGenerics};
use syn::{
    parse::Parse, ConstParam, Error, Expr, GenericArgument, GenericParam, Generics, Ident,
    Lifetime, LifetimeParam, Result, Type, TypeParam, WhereClause,
};

mod linked_table;
mod split_for_impl;

pub struct GenericsModifier {
    lifetimes: LinkedTable<(LifetimeParam, Option<KnownParam>)>,
    types: LinkedTable<(TypeConstParam, Option<KnownParam>)>,
    pub where_clause: Option<WhereClause>,
}

impl GenericsModifier {
    pub fn new(generics: Generics) -> Result<Self> {
        let mut this = GenericsModifier {
            lifetimes: LinkedTable::new(),
            types: LinkedTable::new(),
            where_clause: generics.where_clause,
        };

        for param in generics.params.into_iter() {
            this.push_param(param, None)?;
        }

        Ok(this)
    }

    pub fn push_param(&mut self, param: GenericParam, is_known: Option<KnownParam>) -> Result<()> {
        match param {
            GenericParam::Lifetime(l) => self
                .lifetimes
                .push(l.lifetime.ident.clone(), (l, is_known))?,
            GenericParam::Const(c) => self
                .types
                .push(c.ident.clone(), (TypeConstParam::Const(c), is_known))?,
            GenericParam::Type(t) => self
                .types
                .push(t.ident.clone(), (TypeConstParam::Type(t), is_known))?,
        }
        Ok(())
    }

    pub fn set_known(&mut self, key: &Ident, value: KnownParam) -> Result<()> {
        match value {
            KnownParam::Lifetime(_) => {
                return set_known(&mut self.lifetimes, key, value);
            }
            KnownParam::Const(_) | KnownParam::Type(_) | KnownParam::UseDefault => {
                return set_known(&mut self.types, key, value);
            }
        }
    }

    pub fn split_for_impl(&self) -> (ImplGenerics, TypeGenerics, &Option<WhereClause>) {
        (ImplGenerics(self), TypeGenerics(self), &self.where_clause)
    }
}

fn set_known<T>(
    table: &mut LinkedTable<(T, Option<KnownParam>)>,
    key: &Ident,
    known: KnownParam,
) -> Result<()> {
    match table.get_mut(key) {
        Some(param) => {
            param.1 = Some(known);
            Ok(())
        }
        None => Err(Error::new_spanned(
            key,
            format!("no generic parameter named `{key}` was found"),
        )),
    }
}

pub enum TypeConstParam {
    Type(TypeParam),
    Const(ConstParam),
}

impl ToTokens for TypeConstParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Type(t) => t.to_tokens(tokens),
            Self::Const(c) => c.to_tokens(tokens),
        }
    }
}

pub enum KnownParam {
    Lifetime(Lifetime),
    Type(Type),
    Const(Expr),
    UseDefault,
}

impl Parse for KnownParam {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg = GenericArgument::parse(input)?;
        match arg {
            GenericArgument::Lifetime(l) => Ok(KnownParam::Lifetime(l)),
            GenericArgument::Const(c) => Ok(KnownParam::Const(c)),
            GenericArgument::Type(t) => Ok(KnownParam::Type(t)),
            _ => Err(Error::new_spanned(
                arg,
                "only supports lifetimes, consts and types",
            )),
        }
    }
}
