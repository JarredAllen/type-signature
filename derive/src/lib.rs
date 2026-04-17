//! Derive macros for `type-signature` crate.

use std::collections::HashSet;

use proc_macro::TokenStream as TokenStream1;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// A struct collecting all info needed for [`derive_type_signature`].
struct TypeSignatureImpl {
    /// The identifier for the target type.
    ident: syn::Ident,
    /// Any generics on the target type.
    generics: syn::Generics,
    /// Constraints upon generic types, if any.
    ///
    /// We can only implement `TypeSignature` if all fields implement `TypeSignature`. Unlike
    /// stdlib derive macros, we only limit the implementation based on field types. But to
    /// minimize the risk of adding a `SomeConcreteTyNotImplementingTypeSignature: TypeSignature`
    /// bound that effectively always turns off the impl, the field constraints are only applied if
    /// a generic type parameter exists, and this field is otherwise an empty `Vec`.
    generic_constraints: Vec<syn::Type>,
    /// The list of variants for this type.
    ///
    /// For a struct, there is only one variant, but an enum may have multiple.
    variants: Vec<TokenStream>,
}
impl TryFrom<DeriveInput> for TypeSignatureImpl {
    type Error = syn::Error;

    fn try_from(ast: DeriveInput) -> syn::Result<Self> {
        let any_generic_tys = ast
            .generics
            .params
            .iter()
            .any(|param| matches!(param, syn::GenericParam::Type(_)));
        let (variants, generic_constraints) = match ast.data {
            syn::Data::Struct(st) => {
                let (field_impls, field_tys) = match st.fields {
                syn::Fields::Unit => (Vec::new(), Vec::new()),
                syn::Fields::Named(fields) => (
                    fields
                        .named
                        .iter()
                        .map(|field| {
                            let name = field.ident.as_ref().ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "Missing field ident"))?.to_string();
                            let ty = &field.ty;
                            Ok(quote!((#name, &<#ty as ::type_signature::TypeSignature>::SIGNATURE)))
                        })
                        .collect::<syn::Result<_>>()?,
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
            syn::Data::Union(_union) => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "TODO: Support deriving for unions",
                ));
            }
        };
        // Only supply generic constraints if there's a generic type.
        let generic_constraints = if any_generic_tys {
            generic_constraints
        } else {
            Vec::new()
        };
        Ok(Self {
            ident: ast.ident,
            generics: ast.generics,
            generic_constraints,
            variants,
        })
    }
}
impl quote::ToTokens for TypeSignatureImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }

    fn to_token_stream(&self) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let ty_name = self.ident.to_string();
        let generic_constraints = &self.generic_constraints;
        let variants = &self.variants;
        let generic_ty_signatures = self.generics.params.iter().filter_map(|param| {
            if let syn::GenericParam::Type(ty) = param {
                let ident = &ty.ident;
                Some(quote!(&<#ident as ::type_signature::TypeSignature>::SIGNATURE))
            } else {
                None
            }
        });
        let const_generic_signatures = self.generics.params.iter().filter_map(|param| {
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
                    &format!("hash_const_{param_ty}"),
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
        }
    }
}

/// Derive macro for `TypeSignature`.
#[proc_macro_derive(TypeSignature)]
pub fn derive_type_signature(input: TokenStream1) -> TokenStream1 {
    let ast = parse_macro_input!(input as DeriveInput);
    match TypeSignatureImpl::try_from(ast) {
        Ok(imp) => quote!(#imp),
        Err(e) => e.into_compile_error(),
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO Remove this test when union support is added
    #[test]
    fn assert_fails_for_union() {
        assert!(
            TypeSignatureImpl::try_from(
                syn::parse2::<DeriveInput>(quote! { union SampleUnion {} }).unwrap()
            )
            .is_err()
        );
    }
}
