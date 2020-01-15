use proc_macro2::*;
use proc_quote::*;
use syn::*;

pub fn derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = input;

    let fields = match get_fields(data) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error(),
    };
    let num_fields = fields.len();

    quote! {
        impl ::liquid::ObjectView for #ident {
            fn size(&self) -> i32 {
                #num_fields as i32
            }

            fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = ::kstring::KStringCow<'k>> + 'k> {
                let mut keys = Vec::with_capacity(#num_fields);
                #(
                    keys.push(::kstring::KStringCow::from_static(stringify!(#fields)));
                )*
                Box::new(keys.into_iter())
            }

            fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ::liquid::ValueView> + 'k> {
                let mut values = Vec::<&dyn ::liquid::ValueView>::with_capacity(#num_fields);
                #(
                    values.push(&self.#fields);
                )*
                Box::new(values.into_iter())
            }

            fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (::kstring::KStringCow<'k>, &'k dyn ::liquid::ValueView)> + 'k> {
                let mut values = Vec::<(::kstring::KStringCow<'k>, &'k dyn ::liquid::ValueView)>::with_capacity(#num_fields);
                #(
                    values.push((
                        ::kstring::KStringCow::from_static(stringify!(#fields)),
                        &self.#fields,
                    ));
                )*
                Box::new(values.into_iter())
            }

            fn contains_key(&self, index: &str) -> bool {
                match index {
                    #(
                        stringify!(#fields) => true,
                    )*
                    _ => false,
                }
            }

            fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ::liquid::ValueView> {
                match index {
                    #(
                        stringify!(#fields) => Some(&self.#fields),
                    )*
                    _ => None,
                }
            }
        }
    }
}

fn get_fields(data: &Data) -> Result<Vec<&Ident>> {
    let fields = match data {
        Data::Struct(data) => &data.fields,
        Data::Enum(data) => {
            return Err(Error::new_spanned(
                data.enum_token,
                "`ObjectView` support for `enum` is unimplemented.",
            ));
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "Unions cannot impl ObjectView.",
            ));
        }
    };

    let fields = match fields {
        Fields::Named(fields) => fields,
        Fields::Unnamed(fields) => {
            return Err(Error::new_spanned(
                fields,
                "`ObjectView` support for tuple-structs is unimplemented.",
            ))
        }
        Fields::Unit => {
            return Err(Error::new_spanned(
                fields,
                "`ObjectView` support for unit-structs is unimplemented.",
            ))
        }
    };

    Ok(fields
        .named
        .iter()
        .map(|field| field.ident.as_ref().expect("Fields are named."))
        .collect())
}
