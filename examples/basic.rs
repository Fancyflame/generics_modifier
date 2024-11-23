use generics_modifier::{GenericsModifier, KnownParam};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Generics, Result};

fn main() -> Result<()> {
    let generics: Generics = parse_quote! {
        <
            'a,
            'b: 'a,
            T: TraitT = u32,
            U: TraitU
        >
    };

    let mut modifier = GenericsModifier::new(generics).unwrap();
    modifier.push_param(parse_quote!('c: 'a), None)?;
    modifier.push_param(
        parse_quote!(V: TraitV = TypeOfV),
        Some(KnownParam::UseDefault),
    )?;

    modifier.set_known(&format_ident!("b"), parse_quote!('static))?;
    modifier.set_known(&format_ident!("U"), parse_quote!(TypeOfU))?;

    let (impl_g, type_g, where_clause) = modifier.split_for_impl();
    let stream = quote! {
        impl #impl_g Foo #type_g
        #where_clause
        {}
    };

    let expected = quote! {
        impl<'a, 'c: 'a, T: TraitT,> Foo<'a, 'static, 'c, T, TypeOfU,> {}
    };

    assert!(stream_eq(&stream, &expected));
    Ok(())
}

fn stream_eq(a: &TokenStream, b: &TokenStream) -> bool {
    a.to_string() == b.to_string()
}
