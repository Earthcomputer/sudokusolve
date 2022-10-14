use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(DynClone, attributes(dyn_clone))]
pub fn derive_dyn_clone(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_span = input.span();
    let data = match input.data {
        Data::Struct(data) => data,
        _ => {
            return syn::Error::new(input_span, "DynClone can only be derived from a struct")
                .to_compile_error()
                .into()
        }
    };
    let target = match input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("dyn_clone"))
    {
        Some(target) => target,
        None => {
            return syn::Error::new(input_span, "expected dyn_clone attribute")
                .to_compile_error()
                .into();
        }
    };
    let target: syn::TraitBound = match target.parse_args() {
        Ok(target) => target,
        Err(err) => return err.to_compile_error().into(),
    };

    let type_name = input.ident;
    let generics = input.generics;

    let clone = match data.fields {
        Fields::Unit => quote!(Self),
        Fields::Unnamed(fields) => {
            let numbers: Vec<_> = (0..fields.unnamed.len()).map(|i| quote!(#i)).collect();
            quote!(Self(#(Clone::clone(&self.#numbers)),*))
        }
        Fields::Named(fields) => {
            let names: Vec<_> = fields
                .named
                .into_iter()
                .map(|field| field.ident.unwrap())
                .collect();
            quote!(Self{#(#names: Clone::clone(&self.#names)),*})
        }
    };

    (quote! {
        impl #generics crate::DynClone<dyn #target> for #type_name #generics {
            fn dyn_clone(&self) -> Box<dyn #target> {
                Box::new(#clone)
            }
        }
    })
    .into()
}
