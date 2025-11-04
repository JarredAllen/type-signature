use std::collections::HashSet;

use proc_macro::TokenStream as TokenStream1;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TypeSignature)]
pub fn derive_type_signature(input: TokenStream1) -> TokenStream1 {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let ty_name = ident.to_string();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let where_clause = where_clause.map(|clause| &clause.predicates);
    let generic_ty_signatures = ast.generics.params.iter().filter_map(|param| {
        if let syn::GenericParam::Type(ty) = param {
            Some(quote!(&<#ty as ::type_signature::TypeSignature>::SIGNATURE))
        } else {
            None
        }
    });
    let const_generic_signatures = ast.generics.params.iter().filter_map(|param| {
        if let syn::GenericParam::Const(const_param) = param {
            let param_ty = match &const_param.ty {
                syn::Type::Path(syn::TypePath { qself: None, path }) => {
                    if let Some(ident) = path.get_ident() {
                        ident.to_string()
                    } else {
                        todo!("Support non-identifier types");
                    }
                }
                _ => todo!("Support non-identifier types"),
            };
            let hash_fn_name = syn::Ident::new(
                &format!("hash_const_{}", param_ty),
                proc_macro2::Span::call_site(),
            );
            let param_val = &const_param.ident;
            let param_name = const_param.ident.to_string();
            Some(quote! { const {
                let mut acc = ::type_signature::__macro_export::hash_str(#param_name);
                ::type_signature::__macro_export::mix_values(
                    &mut acc,
                    ::type_signature::__macro_export::#hash_fn_name(#param_val)
                );
                acc
            }})
        } else {
            None
        }
    });
    let (variants, generic_constraints) = match ast.data {
        syn::Data::Struct(st) => {
            let (field_impls, field_tys) = match st.fields {
                syn::Fields::Unit => (Vec::new(), Vec::new()),
                syn::Fields::Named(fields) => (
                    fields
                        .named
                        .iter()
                        .map(|field| {
                            let name = field.ident.as_ref().unwrap().to_string();
                            let ty = &field.ty;
                            quote!((#name, &<#ty as ::type_signature::TypeSignature>::SIGNATURE))
                        })
                        .collect(),
                    fields.named.iter().map(|field| field.ty.clone()).collect(),
                ),
                syn::Fields::Unnamed(fields) => (
                    fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| {
                            let name = idx.to_string();
                            let ty = &field.ty;
                            quote!((#name, &<#ty as ::type_signature::TypeSignature>::SIGNATURE))
                        })
                        .collect(),
                    fields
                        .unnamed
                        .iter()
                        .map(|field| field.ty.clone())
                        .collect(),
                ),
            };
            let variants = vec![quote!(("", &[ #( #field_impls),* ] ))];
            (variants, field_tys)
        }
        syn::Data::Enum(en) => {
            let (variants, per_variant_field_tys) = en
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = variant.ident.to_string();
                    let field_impls = variant
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| {
                            let name = field
                                .ident
                                .as_ref()
                                .map_or_else(|| idx.to_string(), syn::Ident::to_string);
                            let ty = &field.ty;
                            quote!((#name, &<#ty as ::type_signature::TypeSignature>::SIGNATURE))
                        })
                        .collect::<Vec<_>>();
                    let variant_impl = quote!((#variant_name, &[ #( #field_impls ),* ] ));
                    let field_tys: Vec<_> = variant
                        .fields
                        .iter()
                        .map(|field| field.ty.clone())
                        .collect();
                    (variant_impl, field_tys)
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();
            let field_tys = per_variant_field_tys
                .into_iter()
                .flatten()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            (variants, field_tys)
        }
        syn::Data::Union(_union) => (
            vec![syn::Error::new(
                proc_macro2::Span::call_site(),
                "TODO: Support deriving for unions",
            )
            .to_compile_error()],
            vec![],
        ),
    };
    // Only supply generic constraints if there's a generic type.
    let generic_constraints = if generic_ty_signatures.clone().next().is_some() {
        generic_constraints
    } else {
        Vec::new()
    };
    quote! {
        impl #impl_generics ::type_signature::TypeSignature for #ident #ty_generics
            where
            #where_clause
            #( #generic_constraints: TypeSignature ),*
        {
            const SIGNATURE: ::type_signature::TypeSignatureHasher = ::type_signature::TypeSignatureHasher {
                ty_name: #ty_name,
                ty_generics: &[ #( #generic_ty_signatures ),* ],
                const_generic_hashes: &[ #( #const_generic_signatures ),* ],
                variants: &[ #( #variants ),* ],
            };
        }
    }.into()
}
