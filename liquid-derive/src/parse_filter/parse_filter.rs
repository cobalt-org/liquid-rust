use super::*;

/// Generates implementation of `ParseFilter`.
fn generate_parse_filter(filter_parser: &ParseFilter) -> Result<TokenStream> {
    let ParseFilterMeta {
        parameters_struct_name,
        filter_struct_name,
        ..
    } = &filter_parser.meta;

    let filter_struct_name = filter_struct_name.as_ref().map_err(|err| err.clone())?;

    let impl_parse_filter = filter_parser.generate_impl(quote! { ::liquid::compiler::ParseFilter });

    if let Some(parameters_struct_name) = parameters_struct_name {
        let build_filter_parameters = quote_spanned! {parameters_struct_name.span()=>
            let args = <#parameters_struct_name as ::liquid::compiler::FilterParameters>::from_args(args)?;
        };

        let return_expr = quote_spanned! {filter_struct_name.span()=>
            Ok(::std::boxed::Box::new(<#filter_struct_name as ::std::convert::From<#parameters_struct_name>>::from(args)))
        };

        Ok(quote! {
            #impl_parse_filter {
                fn parse(&self, args: ::liquid::compiler::FilterArguments) -> ::liquid::error::Result<::std::boxed::Box<::liquid::compiler::Filter>> {
                    #build_filter_parameters
                    #return_expr
                }
            }
        })
    } else {
        let return_expr = quote_spanned! {filter_struct_name.span()=>
            ::std::result::Result::Ok(::std::boxed::Box::new(<#filter_struct_name as ::std::default::Default>::default()))
        };
        Ok(quote! {
            #impl_parse_filter {
                fn parse(&self, mut args: ::liquid::compiler::FilterArguments) -> ::liquid::error::Result<::std::boxed::Box<::liquid::compiler::Filter>> {
                    if let ::std::option::Option::Some(arg) = args.positional.next() {
                        return ::std::result::Result::Err(::liquid::error::Error::with_msg("Invalid number of positional arguments")
                            .context("cause", concat!("expected at most 0 positional arguments"))
                        );
                    }
                    if let ::std::option::Option::Some(arg) = args.keyword.next() {
                        return ::std::result::Result::Err(::liquid::error::Error::with_msg(format!("Unexpected named argument `{}`", arg.0)));
                    }

                    #return_expr
                }
            }
        })
    }
}

pub fn derive(input: &DeriveInput) -> TokenStream {
    let filter_parser = match ParseFilter::from_input(input) {
        Ok(filter_parser) => filter_parser,
        Err(err) => return err.to_compile_error(),
    };

    let output = match generate_parse_filter(&filter_parser) {
        Ok(output) => output,
        Err(err) => return err.to_compile_error(),
    };

    output
}
