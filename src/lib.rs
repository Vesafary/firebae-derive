extern crate proc_macro;
use self::proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, Fields, GenericParam, Generics};

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(FirebaseMapValue));
        }
    }
    generics
}

fn error(span: proc_macro2::Span, message: &str) -> proc_macro2::TokenStream {
    syn::Error::new(span, message).into_compile_error()
}

#[proc_macro_derive(AsFirebaseMap)]
pub fn impl_as_firebase_map(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let span = proc_macro2::Span::call_site();

    let name = &ast.ident;
    let generics = add_trait_bounds(ast.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data = match ast.data {
        syn::Data::Struct(data) => data,
        _ => return error(span, "AsFirebaseMap should be called on a struct").into(),
    };

    let fields = match data.fields {
        Fields::Named(fields) => fields.named,
        _ => return error(span, "AsFirebaseMap only works on named fields").into(),
    };

    let inserts = fields.into_iter().map(|f| {
        let name = f.ident.unwrap();

        quote! {
            h.insert(stringify!(#name), &self.#name);
        }
    });

    let mod_name = syn::Ident::new(
        &format!("__impl_as_firebase_map_{}", name),
        proc_macro2::Span::call_site(),
    );

    TokenStream::from(quote! {
        mod #mod_name {
            use firebae_cm::{
                IntoFirebaseMap,
                FirebaseMap,
                FirebaseMapValue,
            };
            use super::{#name};

            impl #impl_generics IntoFirebaseMap for #name #ty_generics #where_clause {
                fn as_map(&self) -> FirebaseMap {
                    let mut h = FirebaseMap::new();
                    #(#inserts)*
                    h
                }
            }
        }
    })
}
