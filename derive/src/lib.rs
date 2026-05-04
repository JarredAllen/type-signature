//! Derive macros for `type-signature` crate.

use std::collections::HashSet;

use proc_macro::TokenStream as TokenStream1;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Path, parse_macro_input};

/// A struct collecting all info needed for [`derive_type_signature`].
struct TypeSignatureImpl {
    /// The identifier for the target type.
    ident: syn::Ident,
    /// Any generics on the target type.
    generics: syn::Generics,
    /// Extra `FieldTy: TypeSignature` bounds derived from the types of the (non-skipped)
    /// fields.
    ///
    /// Used in addition to the unconditional `T: TypeSignature` bound on every generic type
    /// parameter, to cover user-defined generic types whose `TypeSignature` impl carries extra
    /// trait bounds (e.g. `MyWrapper<T> where T: SomeTrait + TypeSignature`). Only populated when
    /// the type has at least one generic type parameter, to avoid adding a `SomeConcreteTy:
    /// TypeSignature` bound that could effectively disable the impl.
    generic_constraints: Vec<syn::Type>,
    /// The list of variants for this type.
    ///
    /// For a struct, there is only one variant, but an enum may have multiple.
    variants: Vec<TokenStream>,
    /// If `Some`, override the name emitted into the signature (from `#[type_signature(rename = "...")]`).
    rename: Option<String>,
    /// The path to use for accessing the `type_signature` crate.
    crate_path: Path,
}
impl TryFrom<DeriveInput> for TypeSignatureImpl {
    type Error = syn::Error;

    fn try_from(ast: DeriveInput) -> syn::Result<Self> {
        let type_attrs = TypeAttrs::parse(&ast.attrs)?;
        let crate_path = type_attrs.crate_path.unwrap_or_else(|| Path {
            leading_colon: Some(syn::token::PathSep(Span::call_site())),
            segments: {
                let mut segments = syn::punctuated::Punctuated::new();
                segments.push(syn::Ident::new("type_signature", Span::call_site()).into());
                segments
            },
        });
        for param in &ast.generics.params {
            if let syn::GenericParam::Const(const_param) = param {
                let is_ident = matches!(
                    &const_param.ty,
                    syn::Type::Path(syn::TypePath { qself: None, path })
                        if path.get_ident().is_some()
                );
                if !is_ident {
                    return Err(syn::Error::new_spanned(
                        &const_param.ty,
                        "TypeSignature derive only supports const generic parameters whose type is a simple identifier (e.g. `usize`, `bool`)",
                    ));
                }
            }
        }
        let any_generic_tys = ast
            .generics
            .params
            .iter()
            .any(|param| matches!(param, syn::GenericParam::Type(_)));
        let (variants, generic_constraints) = match ast.data {
            syn::Data::Struct(st) => {
                let (field_impls, field_tys) = fields_info(&st.fields, &crate_path)?;
                let variants = vec![quote!(("", &[ #( #field_impls ),* ]))];
                (variants, field_tys)
            }
            syn::Data::Enum(en) => {
                let rows = en
                    .variants
                    .iter()
                    .map(|variant| -> syn::Result<_> {
                        let variant_attrs = TypeAttrs::parse(&variant.attrs)?;
                        let variant_name = variant_attrs
                            .rename
                            .unwrap_or_else(|| variant.ident.to_string());
                        let (field_impls, field_tys) = fields_info(&variant.fields, &crate_path)?;
                        let variant_impl = quote!((#variant_name, &[ #( #field_impls ),* ]));
                        Ok((variant_impl, field_tys))
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                let (variants, per_variant_field_tys): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
                let field_tys = per_variant_field_tys
                    .into_iter()
                    .flatten()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                (variants, field_tys)
            }
            syn::Data::Union(un) => un
                .fields
                .named
                .iter()
                .filter_map(|field| {
                    let attrs = match FieldAttrs::parse(&field.attrs) {
                        Ok(a) => a,
                        Err(e) => return Some(Err(e)),
                    };
                    if attrs.skip {
                        return None;
                    }
                    let name = attrs.rename.unwrap_or_else(|| {
                        field
                            .ident
                            .as_ref()
                            .expect("union fields are always named")
                            .to_string()
                    });
                    let ty = &field.ty;
                    let variant = quote!(
                        (#name, &[("", &<#ty as #crate_path::TypeSignature>::SIGNATURE)])
                    );
                    Some(Ok((variant, field.ty.clone())))
                })
                .collect::<syn::Result<Vec<_>>>()?
                .into_iter()
                .unzip(),
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
            rename: type_attrs.rename,
            crate_path,
        })
    }
}
impl quote::ToTokens for TypeSignatureImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream());
    }

    fn to_token_stream(&self) -> TokenStream {
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();
        // Extract the raw predicates (without the leading `where` keyword) so we can merge
        // them with our own `FieldTy: TypeSignature` bounds under a single `where` clause.
        let user_where_predicates: Vec<&syn::WherePredicate> = self
            .generics
            .where_clause
            .as_ref()
            .map(|wc| wc.predicates.iter().collect())
            .unwrap_or_default();
        let ident = &self.ident;
        let ty_name = self
            .rename
            .clone()
            .unwrap_or_else(|| self.ident.to_string());
        let generic_constraints = &self.generic_constraints;
        let variants = &self.variants;
        let crate_path = &self.crate_path;
        // Every generic type parameter is unconditionally bounded by `TypeSignature`.
        // This covers cases where the parameter appears only in `ty_generics` (e.g. empty
        // enums, or structs where every generic-typed field is `#[type_signature(skip)]`).
        let generic_ty_bounds = self.generics.params.iter().filter_map(|param| {
            if let syn::GenericParam::Type(ty) = param {
                let ident = &ty.ident;
                Some(quote!(#ident: #crate_path::TypeSignature))
            } else {
                None
            }
        });
        let generic_ty_signatures = self.generics.params.iter().filter_map(|param| {
            if let syn::GenericParam::Type(ty) = param {
                let ident = &ty.ident;
                Some(quote!(&<#ident as #crate_path::TypeSignature>::SIGNATURE))
            } else {
                None
            }
        });
        let const_generic_signatures = self.generics.params.iter().filter_map(|param| {
            if let syn::GenericParam::Const(const_param) = param {
                let syn::Type::Path(syn::TypePath { qself: None, path }) = &const_param.ty else {
                    unreachable!("validated in TryFrom::try_from")
                };
                let param_ty = path
                    .get_ident()
                    .expect("validated in TryFrom::try_from")
                    .to_string();
                let hash_fn_name =
                    syn::Ident::new(&format!("hash_const_{param_ty}"), Span::call_site());
                let param_val = &const_param.ident;
                let param_name = const_param.ident.to_string();
                Some(quote! { const {
                    let mut acc = #crate_path::__macro_export::hash_str(#param_name);
                    #crate_path::__macro_export::mix_values(
                        &mut acc,
                        #crate_path::__macro_export::#hash_fn_name(#param_val)
                    );
                    acc
                }})
            } else {
                None
            }
        });
        quote! {
            impl #impl_generics #crate_path::TypeSignature for #ident #ty_generics
                where
                    #( #user_where_predicates, )*
                    #( #generic_ty_bounds, )*
                    #( #generic_constraints: #crate_path::TypeSignature ),*
            {
                #![allow(single_use_lifetimes, reason = "Macro-generated code")]
                const SIGNATURE: #crate_path::TypeSignatureHasher = #crate_path::TypeSignatureHasher {
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
#[proc_macro_derive(TypeSignature, attributes(type_signature))]
pub fn derive_type_signature(input: TokenStream1) -> TokenStream1 {
    let ast = parse_macro_input!(input as DeriveInput);
    match TypeSignatureImpl::try_from(ast) {
        Ok(imp) => quote!(#imp),
        Err(e) => e.into_compile_error(),
    }
    .into()
}

/// Build `(field_impl_tokens, field_type)` pairs for every field, covering unit/named/tuple shapes.
///
/// Fields marked `#[type_signature(skip)]` are omitted from both vectors.
fn fields_info(
    fields: &syn::Fields,
    crate_path: &Path,
) -> syn::Result<(Vec<TokenStream>, Vec<syn::Type>)> {
    let rows = fields
        .iter()
        .enumerate()
        .filter_map(|(idx, field)| {
            let attrs = match FieldAttrs::parse(&field.attrs) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            };
            if attrs.skip {
                return None;
            }
            let name = attrs.rename.unwrap_or_else(|| {
                field
                    .ident
                    .as_ref()
                    .map_or_else(|| idx.to_string(), syn::Ident::to_string)
            });
            let ty = &field.ty;
            let impl_tokens = quote!((#name, &<#ty as #crate_path::TypeSignature>::SIGNATURE));
            Some(Ok((impl_tokens, field.ty.clone())))
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(rows.into_iter().unzip())
}

/// Parsed `#[type_signature(...)]` attributes at the type level.
#[derive(Default)]
struct TypeAttrs {
    /// Override the name used in the signature. Lets the signature survive a type rename.
    rename: Option<String>,
    /// The path to the `type_signature` crate if it needs to be overridden.
    crate_path: Option<Path>,
}

impl TypeAttrs {
    fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut out = Self::default();
        for attr in attrs {
            if !attr.path().is_ident("type_signature") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    out.rename = Some(lit.value());
                    Ok(())
                } else if meta.path.is_ident("crate") {
                    let crate_path: Path = meta.value()?.parse()?;
                    out.crate_path = Some(crate_path);
                    Ok(())
                } else {
                    Err(meta.error("unrecognized type_signature attribute {attr:?}"))
                }
            })?;
        }
        Ok(out)
    }
}

/// Parsed `#[type_signature(...)]` attributes at the field level.
#[derive(Default)]
struct FieldAttrs {
    /// Omit this field from the signature entirely.
    skip: bool,
    /// Override the name used for this field in the signature.
    rename: Option<String>,
}

impl FieldAttrs {
    fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut out = Self::default();
        for attr in attrs {
            if !attr.path().is_ident("type_signature") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    out.skip = true;
                    Ok(())
                } else if meta.path.is_ident("rename") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    out.rename = Some(lit.value());
                    Ok(())
                } else {
                    Err(meta.error(
                        "unrecognized type_signature attribute; expected `skip` or `rename = \"...\"`",
                    ))
                }
            })?;
        }
        Ok(out)
    }
}
