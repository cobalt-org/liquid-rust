use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned as _;
use syn::{Attribute, Data, DeriveInput, Error, Generics, Path, Result};

use crate::helpers::{assign_path, assign_str_value, AssignOnce};

pub(crate) mod filter_reflection;
pub(crate) mod parse;

/// Struct that contains information to generate the necessary code for `ParseFilter`.
struct ParseFilter<'a> {
    name: &'a Ident,
    meta: ParseFilterMeta,
    generics: &'a Generics,
}

impl<'a> ParseFilter<'a> {
    /// Generates `impl` declaration of the given trait for the structure
    /// represented by `self`.
    fn generate_impl(&self, trait_name: TokenStream) -> TokenStream {
        let name = &self.name;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        quote! {
            impl #impl_generics #trait_name for #name #ty_generics #where_clause
        }
    }

    /// Asserts that this is an empty struct.
    fn validate_data(data: &Data) -> Result<()> {
        match data {
            Data::Struct(_) => Ok(()),
            Data::Enum(data) => Err(Error::new_spanned(
                data.enum_token,
                "Enums cannot be ParseFilter.",
            )),
            Data::Union(data) => Err(Error::new_spanned(
                data.union_token,
                "Unions cannot be ParseFilter.",
            )),
        }
    }

    /// Searches for `#[filter(...)]` in order to parse `ParseFilterMeta`.
    fn parse_attrs(attrs: &[Attribute]) -> Result<ParseFilterMeta> {
        let mut filter_attrs = attrs.iter().filter(|attr| attr.path().is_ident("filter"));

        match (filter_attrs.next(), filter_attrs.next()) {
            (Some(attr), None) => ParseFilterMeta::from_attr(attr),

            (_, Some(attr)) => Err(Error::new_spanned(
                attr,
                "Found multiple definitions for `filter` attribute.",
            )),

            _ => Err(Error::new(
                Span::call_site(),
                "Cannot find `filter` attribute in target struct. Have you tried adding `#[parser(name=\"...\", description=\"...\", parameters(...), parsed(...))]`?",
            )),
        }
    }

    /// Tries to create a new `ParseFilter` from the given `DeriveInput`
    fn from_input(input: &'a DeriveInput) -> Result<Self> {
        let DeriveInput {
            attrs,
            data,
            ident,
            generics,
            ..
        } = input;

        Self::validate_data(data)?;
        let meta = Self::parse_attrs(attrs)?;

        Ok(ParseFilter {
            name: ident,
            meta,
            generics,
        })
    }
}

/// Struct that contains information parsed in `#[filter(...)]` attribute.
struct ParseFilterMeta {
    filter_name: Result<String>,
    filter_description: Result<String>,
    parameters_struct_name: Option<Path>,
    filter_struct_name: Result<Path>,
}

impl ParseFilterMeta {
    /// Tries to create a new `ParseFilterMeta` from the given `Attribute`
    fn from_attr(attr: &Attribute) -> Result<Self> {
        let mut name = AssignOnce::Unset;
        let mut description = AssignOnce::Unset;
        let mut parameters = AssignOnce::Unset;
        let mut parsed = AssignOnce::Unset;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                assign_str_value(&mut name, attr, "name", &meta)?;
            } else if meta.path.is_ident("description") {
                assign_str_value(&mut description, attr, "description", &meta)?;
            } else if meta.path.is_ident("parameters") {
                assign_path(&mut parameters, attr, "parameters", &meta)?;
            } else if meta.path.is_ident("parsed") {
                assign_path(&mut parsed, attr, "parsed", &meta)?;
            } else {
                return Err(Error::new(
                    attr.span(),
                    format!(
                        "unknown `{}` parameter attribute",
                        meta.path.to_token_stream()
                    ),
                ));
            }
            Ok(())
        })?;

        let filter_name = name.unwrap_or_err(|| Error::new_spanned(
            attr,
            "FilterReflection does not have a name. Have you tried `#[filter(name=\"...\", description=\"...\", parameters(...), parsed(...))]`?",
        ));
        let filter_description = description.unwrap_or_err(|| Error::new_spanned(
            attr,
            "FilterReflection does not have a description. Have you tried `#[filter(name=\"...\", description=\"...\", parameters(...), parsed(...))]`?",
        ));
        let parameters_struct_name = parameters.into_option();
        let filter_struct_name = parsed.unwrap_or_err(|| Error::new_spanned(
            attr,
            "ParseFilter does not have a Filter to return. Have you tried `#[filter(name=\"...\", description=\"...\", parameters(...), parsed(...))]`?",
        ));

        Ok(ParseFilterMeta {
            filter_name,
            filter_description,
            parameters_struct_name,
            filter_struct_name,
        })
    }
}
