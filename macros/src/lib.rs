use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, Path, Type, meta::ParseNestedMeta, parse_macro_input, parse_quote,
};

#[proc_macro_derive(FieldEnum, attributes(field))]
pub fn field_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let enum_name = format_ident!("{}Field", struct_name);
    let value_enum_name = format_ident!("{}Value", struct_name);

    let mut derives: Vec<Path> = vec![
        parse_quote!(Clone),
        parse_quote!(Copy),
        parse_quote!(PartialEq),
        parse_quote!(Eq),
        parse_quote!(Hash),
    ];
    let mut value_derives: Vec<Path> = vec![parse_quote!(Clone), parse_quote!(PartialEq)];
    for attr in input.attrs {
        if attr.path().is_ident("field") {
            attr.parse_nested_meta(|meta| {
                let ParseNestedMeta { path, .. } = &meta;
                if path.is_ident("derive") {
                    meta.parse_nested_meta(|meta| {
                        derives.push(meta.path);
                        Ok(())
                    })
                } else if path.is_ident("derive_value") {
                    meta.parse_nested_meta(|meta| {
                        value_derives.push(meta.path);
                        Ok(())
                    })
                } else {
                    Ok(())
                }
            })
            .unwrap();
        }
    }

    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => {
                return syn::Error::new_spanned(struct_name, "Only named fields are supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(struct_name, "Only structs are supported")
                .to_compile_error()
                .into();
        }
    };

    let mut enum_variants = Vec::new();
    let mut field_types = Vec::new();
    let mut field_setters = Vec::new();
    let mut field_getters = Vec::new();
    let mut try_into_impls = Vec::new();
    let mut seen_field_types = HashSet::new();

    for field in fields {
        let field_ty = &field.ty;

        // This field can only be replaced as a whole,
        // i.e. no nested field access.
        // Primitive types are treated this way,
        // but other fields can be tagged to be treated
        // this way using `#[field(swap_only)]`.
        let mut swap_only = is_primitive(field_ty);

        let mut skip = false;
        for attr in field.attrs {
            if attr.path().is_ident("field") {
                attr.parse_nested_meta(|meta| {
                    let ParseNestedMeta { path, .. } = &meta;
                    if path.is_ident("skip") {
                        skip = true;
                    } else if path.is_ident("swap_only") {
                        swap_only = true;
                    }
                    Ok(())
                })
                .unwrap();
            }
        }

        if skip {
            continue;
        }

        let field_name = field.ident.unwrap();
        let variant_name = format_ident!("{}", field_name.to_string().to_case(Case::Pascal));

        let variant = if swap_only {
            quote! {
                #variant_name
            }
        } else {
            quote! {
                #variant_name(<#field_ty as Fielded>::Field)
            }
        };
        enum_variants.push(variant.clone());

        let inner_type = if swap_only {
            quote! { #field_ty }
        } else {
            quote! { <#field_ty as Fielded>::FieldValue }
        };

        let setter = if swap_only {
            quote! {
                Self::Field::#variant_name => self.#field_name = value.try_into().map_err(|mut err: SetFieldError| {
                    err.field = stringify!(#variant_name);
                    err
                })?
            }
        } else {
            quote! {
                Self::Field::#variant_name(inner) => {
                    let inner_value: #inner_type = value.try_into().map_err(|mut err: SetFieldError| {
                        err.field = stringify!(#variant_name);
                        err
                    })?;
                    self.#field_name.set_field(inner, inner_value)?
                }
            }
        };

        let getter = if swap_only {
            quote! {
                Self::Field::#variant_name => {
                    self.#field_name.clone().into()
                }
            }
        } else {
            quote! {
                Self::Field::#variant_name(inner) => {
                    self.#field_name.get_field(inner).into()
                }
            }
        };

        field_setters.push(setter);
        field_getters.push(getter);

        let ty_string = field_ty.to_token_stream().to_string();
        if seen_field_types.contains(&ty_string) {
            continue;
        }

        let ty_ident = match field_ty {
            Type::Path(type_path) => {
                // get last segment like `String` or `u32`
                let base = &type_path.path.segments.last().unwrap().ident;
                format_ident!("{}", base.to_string().to_case(Case::Pascal))
            }
            _ => format_ident!("UnknownType"),
        };

        field_types.push(quote! { #ty_ident(#inner_type) });
        seen_field_types.insert(ty_string);

        try_into_impls.push(quote! {
            impl TryInto<#inner_type> for #value_enum_name {
                type Error = SetFieldError;
                fn try_into(self) -> Result<#inner_type, Self::Error> {
                    match self {
                        Self::#ty_ident(value) => Ok(value),
                        other => Err(SetFieldError {
                            field: "",
                            received: format!("{:?}", other),
                            expected: std::any::type_name::<#inner_type>()
                        })
                    }
                }
            }

            impl From<#inner_type> for #value_enum_name {
                fn from(value: #inner_type) -> Self {
                    Self::#ty_ident(value)
                }
            }
        });
    }

    let derive_attr = if !derives.is_empty() {
        Some(quote! { #[derive( #(#derives),* )] })
    } else {
        None
    };

    let derive_value_attr = if !value_derives.is_empty() {
        Some(quote! { #[derive( #(#value_derives),* )] })
    } else {
        None
    };

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #derive_attr
        pub enum #enum_name {
            #(#enum_variants,)*
        }

        #derive_value_attr
        pub enum #value_enum_name {
            #(#field_types,)*
        }

        #(#try_into_impls)*

        impl Fielded for #struct_name {
            type Field = #enum_name;
            type FieldValue = #value_enum_name;

            fn set_field<V: Into<Self::FieldValue>>(&mut self, field: Self::Field, value: V) -> Result<(), SetFieldError> {
                let value: Self::FieldValue = value.into();
                match field {
                    #(#field_setters,)*
                }
                Ok(())
            }

            fn get_field(&self, field: &Self::Field) -> Self::FieldValue {
                match field {
                    #(#field_getters,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_primitive(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() {
            let ident = &type_path.path.segments.last().unwrap().ident;
            let name = ident.to_string();

            matches!(
                name.as_str(),
                "u8" | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "usize"
                    | "isize"
                    | "bool"
                    | "char"
                    | "f32"
                    | "f64"
                    | "String"
                    | "str"
            )
        } else {
            false
        }
    } else {
        false
    }
}
