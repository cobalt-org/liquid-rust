use proc_macro2::*;
use proc_quote::*;
use syn::*;

pub fn derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::liquid::ValueView for #ident #ty_generics #where_clause {
            fn as_debug(&self) -> &dyn ::std::fmt::Debug {
                self
            }

            fn render(&self) -> ::liquid::model::DisplayCow<'_> {
                ::liquid::model::DisplayCow::Owned(Box::new(::liquid::model::ObjectRender::new(self)))
            }
            fn source(&self) -> ::liquid::model::DisplayCow<'_> {
                ::liquid::model::DisplayCow::Owned(Box::new(::liquid::model::ObjectSource::new(self)))
            }
            fn type_name(&self) -> &'static str {
                "object"
            }
            fn query_state(&self, state: ::liquid::model::State) -> bool {
                match state {
                    ::liquid::model::State::Truthy => true,
                    ::liquid::model::State::DefaultValue |
                    ::liquid::model::State::Empty |
                    ::liquid::model::State::Blank => self.size() == 0,
                }
            }

            fn to_kstr(&self) -> ::kstring::KStringCow<'_> {
                let s = ::liquid::model::ObjectRender::new(self).to_string();
                ::kstring::KStringCow::from_string(s)
            }
            fn to_value(&self) -> ::liquid::model::Value {
                ::liquid::model::to_value(self).unwrap()
            }

            fn as_object(&self) -> Option<&dyn ::liquid::ObjectView> {
                Some(self)
            }
        }
    }
}

pub fn core_derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::liquid_core::ValueView for #ident #ty_generics #where_clause {
            fn as_debug(&self) -> &dyn ::std::fmt::Debug {
                self
            }

            fn render(&self) -> ::liquid_core::model::DisplayCow<'_> {
                ::liquid_core::model::DisplayCow::Owned(Box::new(::liquid_core::model::ObjectRender::new(self)))
            }
            fn source(&self) -> ::liquid_core::model::DisplayCow<'_> {
                ::liquid_core::model::DisplayCow::Owned(Box::new(::liquid_core::model::ObjectSource::new(self)))
            }
            fn type_name(&self) -> &'static str {
                "object"
            }
            fn query_state(&self, state: ::liquid_core::model::State) -> bool {
                match state {
                    ::liquid_core::model::State::Truthy => true,
                    ::liquid_core::model::State::DefaultValue |
                    ::liquid_core::model::State::Empty |
                    ::liquid_core::model::State::Blank => self.size() == 0,
                }
            }

            fn to_kstr(&self) -> ::kstring::KStringCow<'_> {
                let s = ::liquid_core::model::ObjectRender::new(self).to_string();
                ::kstring::KStringCow::from_string(s)
            }
            fn to_value(&self) -> ::liquid_core::model::Value {
                ::liquid_core::model::to_value(self).unwrap()
            }

            fn as_object(&self) -> Option<&dyn ::liquid_core::ObjectView> {
                Some(self)
            }
        }
    }
}
