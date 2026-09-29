extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

struct StructAttrs {
    description: Option<String>,
}

struct FieldAttrs {
    rename: Option<String>,
    description: Option<String>,
    optional: Option<bool>,
    skip: bool,
    minimum: Option<f64>,
    maximum: Option<f64>,
    variants: Vec<String>,
    serde_json: bool,
}

#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_json_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_derive_json_schema(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_derive_json_schema(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (struct_attrs, struct_doc) = parse_struct_attrs(&input.attrs)?;
    let struct_desc = struct_attrs.description.unwrap_or(struct_doc);

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "#[derive(::anylm::Schema)] supports only structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "#[derive(::anylm::Schema)] supports only structs",
            ));
        }
    };

    let mut property_statements = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let (field_attrs, field_doc) = parse_field_attrs(&field.attrs)?;

        if field_attrs.skip {
            continue;
        }

        let prop_name = field_attrs
            .rename
            .unwrap_or_else(|| field_ident.to_string());
        let field_desc = field_attrs.description.unwrap_or(field_doc);

        let is_type_option = is_option_type(field_ty);

        let field_schema_expr = if field_attrs.serde_json {
            quote! {
                let mut field_schema = <String as ::anylm::IntoSchema>::schema();
                field_schema.optional = Some(#is_type_option);
            }
        } else {
            quote! {
                let mut field_schema = <#field_ty as ::anylm::IntoSchema>::schema();
            }
        };

        let desc_override = if !field_desc.is_empty() {
            quote! { field_schema.description = Some(#field_desc.to_string()); }
        } else {
            quote! {}
        };

        let min_call = if let Some(min) = field_attrs.minimum {
            quote! { field_schema = field_schema.minimum(#min); }
        } else {
            quote! {}
        };

        let max_call = if let Some(max) = field_attrs.maximum {
            quote! { field_schema = field_schema.maximum(#max); }
        } else {
            quote! {}
        };

        let variants_call = if !field_attrs.variants.is_empty() {
            let vars = &field_attrs.variants;
            quote! {
                #( field_schema = field_schema.variant(#vars); )*
            }
        } else {
            quote! {}
        };

        let required_logic = if let Some(is_opt) = field_attrs.optional {
            let is_req = !is_opt;
            quote! { let is_req = #is_req; }
        } else {
            quote! {
                let is_req = !field_schema.optional.unwrap_or(false);
            }
        };

        property_statements.push(quote! {
            {
                #field_schema_expr
                #desc_override
                #min_call
                #max_call
                #variants_call
                #required_logic
                schema = schema.property(#prop_name, field_schema, is_req);
            }
        });
    }

    let expanded = quote! {
        impl #impl_generics ::anylm::IntoSchema for #struct_ident #ty_generics #where_clause {
            fn schema() -> ::anylm::JsonSchema {
                let mut schema = ::anylm::JsonSchema::object(#struct_desc);
                #(#property_statements)*
                schema
            }
        }
    };

    Ok(expanded)
}

fn parse_struct_attrs(attrs: &[syn::Attribute]) -> syn::Result<(StructAttrs, String)> {
    let mut struct_attrs = StructAttrs { description: None };
    let mut doc_comments = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    let text = lit_str.value().trim().to_string();
                    if !text.is_empty() {
                        doc_comments.push(text);
                    }
                }
            }
        } else if attr.path().is_ident("schema") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("description") || meta.path.is_ident("descr") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    struct_attrs.description = Some(value.value());
                }
                Ok(())
            })?;
        }
    }

    Ok((struct_attrs, doc_comments.join(" ")))
}

fn parse_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<(FieldAttrs, String)> {
    let mut field_attrs = FieldAttrs {
        rename: None,
        description: None,
        optional: None,
        skip: false,
        minimum: None,
        maximum: None,
        variants: Vec::new(),
        serde_json: false,
    };
    let mut doc_comments = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    let text = lit_str.value().trim().to_string();
                    if !text.is_empty() {
                        doc_comments.push(text);
                    }
                }
            }
        } else if attr.path().is_ident("schema") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("serde_json") {
                    field_attrs.serde_json = true;
                } else if meta.path.is_ident("rename") || meta.path.is_ident("name") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    field_attrs.rename = Some(value.value());
                } else if meta.path.is_ident("description") || meta.path.is_ident("descr") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    field_attrs.description = Some(value.value());
                } else if meta.path.is_ident("optional") {
                    field_attrs.optional = Some(true);
                } else if meta.path.is_ident("required") {
                    field_attrs.optional = Some(false);
                } else if meta.path.is_ident("skip") {
                    field_attrs.skip = true;
                } else if meta.path.is_ident("min") || meta.path.is_ident("minimum") {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = expr {
                        match lit {
                            syn::Lit::Float(f) => field_attrs.minimum = Some(f.base10_parse()?),
                            syn::Lit::Int(i) => field_attrs.minimum = Some(i.base10_parse()?),
                            _ => {}
                        }
                    }
                } else if meta.path.is_ident("max") || meta.path.is_ident("maximum") {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = expr {
                        match lit {
                            syn::Lit::Float(f) => field_attrs.maximum = Some(f.base10_parse()?),
                            syn::Lit::Int(i) => field_attrs.maximum = Some(i.base10_parse()?),
                            _ => {}
                        }
                    }
                } else if meta.path.is_ident("variants") || meta.path.is_ident("variant") {
                    if meta.input.peek(syn::Token![=]) {
                        let value: syn::Expr = meta.value()?.parse()?;
                        if let syn::Expr::Array(arr) = value {
                            for expr in arr.elems {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(s),
                                    ..
                                }) = expr
                                {
                                    field_attrs.variants.push(s.value());
                                }
                            }
                        } else if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = value
                        {
                            field_attrs.variants.push(s.value());
                        }
                    } else if meta.input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let lit_strs =
                            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(&content)?;
                        for lit in lit_strs {
                            field_attrs.variants.push(lit.value());
                        }
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok((field_attrs, doc_comments.join(" ")))
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_none() {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Option";
            }
        }
    }
    false
}
