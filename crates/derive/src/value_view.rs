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
            fn render(&self) -> ::liquid::value::DisplayCow<'_> {
                ::liquid::value::DisplayCow::Owned(Box::new(::liquid::value::ObjectRender::new(self)))
            }
            fn source(&self) -> ::liquid::value::DisplayCow<'_> {
                ::liquid::value::DisplayCow::Owned(Box::new(::liquid::value::ObjectSource::new(self)))
            }
            fn type_name(&self) -> &'static str {
                "object"
            }
            fn query_state(&self, state: ::liquid::value::State) -> bool {
                match state {
                    ::liquid::value::State::Truthy => true,
                    ::liquid::value::State::DefaultValue |
                    ::liquid::value::State::Empty |
                    ::liquid::value::State::Blank => self.size() == 0,
                }
            }

            fn to_kstr(&self) -> ::kstring::KStringCow<'_> {
                let s = ::liquid::value::ObjectRender::new(self).to_string();
                ::kstring::KStringCow::from_string(s)
            }
            fn to_value(&self) -> ::liquid::value::Value {
                ::liquid::value::to_value(self).unwrap()
            }

            fn as_object(&self) -> Option<&dyn ::liquid::ObjectView> {
                Some(self)
            }
        }
    }
}
