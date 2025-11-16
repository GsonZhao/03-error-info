use darling::{
    FromDeriveInput, FromVariant,
    ast::{Data, Fields, Style},
    util::Ignored,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ErrorData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<EnumVariants, ()>,
    app_type: syn::Type,
    prefix: String,
}

#[allow(dead_code)]
#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumVariants {
    ident: syn::Ident,
    fields: Fields<Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

pub fn process_error_info(input: syn::DeriveInput) -> TokenStream {
    let ErrorData {
        ident: name,
        generics,
        data: Data::Enum(data),
        app_type,
        prefix,
    } = ErrorData::from_derive_input(&input).expect("Failed to parse error data")
    else {
        panic!("Only enum variants are supported");
    };

    //for each variant, we need to generate a new error info struct
    // #name::#ident(_) => { // code to new ErrorInfo}

    let code = data
        .iter()
        .map(|v| {
            let EnumVariants {
                ident,
                fields,
                code,
                app_code,
                client_msg,
            } = v;
            let code = format!("{}{}", prefix, code);

            let variant_code = match fields.style {
                Style::Unit => { quote! { #name::#ident } },
                Style::Tuple => { quote! { #name::#ident(_) } },
                Style::Struct => { quote! { #name::#ident{..} } },
            };

            quote! {
                #variant_code => {
                    ErrorInfo::new(#app_code, #code, #client_msg, self)
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        use error_code::ToErrorInfo as _;
        use error_code::ErrorInfo;
        impl #generics ToErrorInfo for #name #generics {
            type T = #app_type;
            fn to_error_info(&self) -> ErrorInfo<Self::T> {
                match self {
                    #(#code)*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_struct() {
        let input = r#"
            #[derive(Debug,thiserror::Error,ToErrorInfo)]
            #[error_info(app_type="StatusCode",prefix="01")]
            pub enum MyError {
                #[error("Invalid Command: {0}")]
                #[error_info(code ="IC", app_code = "400")]
                InvalidCommand(String),

                #[error("Invalid_argument: {0}")]
                #[error_info(code="IA", app_code = "400",client_msg="friendly msg")]
                InvalidArgument(String),

                #[error("{0}")]
                #[error_info(code="RE", app_code = "500")]
                RespError(#[from] std::io::Error),
            }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = ErrorData::from_derive_input(&parsed).unwrap();

        println!("{:#?}", info);

        let code = process_error_info(parsed);
        println!("{}", code);
    }
}
