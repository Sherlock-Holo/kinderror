//! Generate io::Error style error.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Ident, Meta, Token, Type, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Generate io::Error style error.
///
/// # Example
///
/// ```rust
/// use kinderror::KindError;
/// use std::error::Error as _;
///
/// #[derive(KindError, Debug, Eq, PartialEq)]
/// #[kind_error(
///     source = "std::io::Error",
///     source_fn = true,
///     new_vis = "pub",
///     name = "Error",
///     type_vis = "pub",
///     kind_fn_vis = "pub",
///     display = "hey, error kind: {kind:?}, source: {source}",
///     origin_fn_vis = "pub(crate)"
/// )]
/// enum ErrorKind {
///     First,
///     Second,
/// }
///
/// let err = Error::new(ErrorKind::First, std::io::Error::other("first error"));
/// assert_eq!(*err.kind(), ErrorKind::First);
/// assert!(err.source().is_some());
/// ```
///
/// # Attributes
///
/// - `source`: (required) source error type, e.g. `"std::io::Error"`
/// - `source_fn`: (default: true) whether to implement the `Error::source()` method. Set to `false` when the source type does not implement `dyn ::core::error::Error + 'static`
/// - `new_vis`: (default: inherited) visibility of the constructor, e.g. `"pub"`
/// - `name`: (default: "Error") name of the generated error struct
/// - `type_vis`: (default: inherited) visibility of the struct
/// - `kind_fn_vis`: (default: pub) visibility of the `kind()` method
/// - `origin_fn_vis`: (default: pub) visibility of the `origin()` method
/// - `display`: (default: "error kind: {kind:?}, source: {source:?}") custom Display format. Supports placeholders `{kind}` and `{source}`, users can freely specify format specifiers
#[proc_macro_derive(KindError, attributes(kind_error))]
pub fn kind_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    kind_error_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}

struct KindErrorAttrs {
    source: Option<Type>,
    new_vis: Option<Visibility>,
    name: Option<String>,
    type_vis: Option<Visibility>,
    kind_fn_vis: Option<Visibility>,
    origin_fn_vis: Option<Visibility>,
    source_fn: bool,
    display: Option<String>,
}

impl Parse for KindErrorAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = KindErrorAttrs {
            source: None,
            new_vis: None,
            name: None,
            type_vis: None,
            kind_fn_vis: None,
            origin_fn_vis: None,
            source_fn: true,
            display: None,
        };

        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "source" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.source = Some(syn::parse_str::<Type>(&lit_str.value())?);
                }
                "new_vis" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.new_vis = Some(syn::parse_str::<Visibility>(&lit_str.value())?);
                }
                "name" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.name = Some(lit_str.value());
                }
                "type_vis" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.type_vis = Some(syn::parse_str::<Visibility>(&lit_str.value())?);
                }
                "kind_fn_vis" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.kind_fn_vis = Some(syn::parse_str::<Visibility>(&lit_str.value())?);
                }
                "origin_fn_vis" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.origin_fn_vis = Some(syn::parse_str::<Visibility>(&lit_str.value())?);
                }
                "source_fn" => {
                    let lit_bool = input.parse::<syn::LitBool>()?;
                    attrs.source_fn = lit_bool.value();
                }
                "display" => {
                    let lit_str = input.parse::<syn::LitStr>()?;
                    attrs.display = Some(lit_str.value());
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("unknown attribute key: {}", key),
                    ));
                }
            }

            // Handle comma separation
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(attrs)
    }
}

fn parse_kind_error_attrs(attrs: &[Attribute]) -> syn::Result<KindErrorAttrs> {
    let kind_error_attr = attrs.iter().find(|attr| attr.path().is_ident("kind_error"));

    if let Some(attr) = kind_error_attr {
        match &attr.meta {
            Meta::List(meta_list) => {
                let attrs: KindErrorAttrs = meta_list.parse_args()?;
                Ok(attrs)
            }
            _ => Err(syn::Error::new_spanned(
                attr,
                "kind_error attribute must be in the form #[kind_error(...)]",
            )),
        }
    } else {
        // Return empty attributes, default values will be used
        Ok(KindErrorAttrs {
            source: None,
            new_vis: None,
            name: None,
            type_vis: None,
            kind_fn_vis: None,
            origin_fn_vis: None,
            source_fn: true,
            display: None,
        })
    }
}

fn kind_error_impl(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    match &input.data {
        Data::Enum(_) => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "This macro only supports Enum, not for other types",
            ));
        }
    };

    let attrs = parse_kind_error_attrs(&input.attrs)?;

    let source_type = attrs
        .source
        .ok_or_else(|| syn::Error::new_spanned(&input, "source attribute is required"))?;
    let kind_type = &input.ident;
    let new_vis = attrs.new_vis.unwrap_or(Visibility::Inherited);
    let type_vis = attrs.type_vis.unwrap_or(Visibility::Inherited);
    let kind_fn_vis = attrs
        .kind_fn_vis
        .unwrap_or(Visibility::Public(Default::default()));
    let origin_fn_vis = attrs
        .origin_fn_vis
        .unwrap_or(Visibility::Public(Default::default()));
    let name_str = attrs.name.as_deref().unwrap_or("Error");
    let name = Ident::new(name_str, input.ident.span());

    let source_method = if attrs.source_fn {
        quote! {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
                Some(&self.source)
            }
        }
    } else {
        quote! {}
    };

    // Handle Display implementation
    let display_impl = if let Some(display_format) = attrs.display {
        // Use the user-provided formatting template directly
        quote! {
            impl ::core::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let kind = &self.kind;
                    let source = &self.source;
                    write!(f, #display_format)
                }
            }
        }
    } else {
        // Use default format
        quote! {
            impl ::core::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "error kind: {:?}, source: {:?}", self.kind, self.source)
                }
            }
        }
    };

    let expand = quote! {
        #[derive(::core::fmt::Debug)]
        #type_vis struct #name {
            kind: #kind_type,
            source: #source_type,
        }

        impl #name {
            #new_vis fn new(kind: #kind_type, source: #source_type) -> Self {
                Self { kind, source }
            }

            #kind_fn_vis fn kind(&self) -> &#kind_type {
                &self.kind
            }

            #origin_fn_vis fn origin(&self) -> &#source_type {
                &self.source
            }
        }

        #display_impl

        impl ::core::error::Error for #name {
            #source_method
        }
    };

    Ok(expand.into())
}
