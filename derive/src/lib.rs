use std::{fs, io};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, ItemMod, parse_macro_input};

use darling::{FromDeriveInput, FromMeta, ast::NestedMeta};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(aseprite), supports(struct_unit))]
struct MyDeriveOpts {
    /// #[aseprite(file = "...")]
    file: String,
}

#[proc_macro_derive(MyDerive, attributes(aseprite))]
pub fn my_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match MyDeriveOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &input.ident;
    let file = opts.file;

    let expanded = quote! {
        impl #ident {
            pub fn file() -> &'static str {
                #file
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(MyOtherDerive, attributes(aseprite))]
pub fn my_other_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match MyDeriveOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &input.ident;
    let file = opts.file;
    let expanded = quote! {
        struct WeshWesh;

        impl WeshWesh {
            pub fn file() -> &'static str {
                #file
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(Debug, FromMeta)]
struct MacroArgs {
    /// #[aseprite(file = "...")]
    file: String,
}

#[proc_macro_attribute]
pub fn asepritee(attr: TokenStream, item: TokenStream) -> TokenStream {
    let meta = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };
    let args = match MacroArgs::from_list(&meta) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let mut module = parse_macro_input!(item as ItemMod);
    let span = module.ident.span();
    let (_, items) = match &mut module.content {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(module, "aseprite macro only supports inline modules")
                .to_compile_error()
                .into();
        }
    };

    let file_path = format!("assets/{}", args.file);
    let ase_bytes = match fs::read(&file_path).map_err(|e| {
        darling::Error::custom(format!("I/O error for file {file_path}: {e}")).with_span(&span)
    }) {
        Ok(bytes) => bytes,
        Err(e) => return e.write_errors().into(),
    };
    let raw = match aseprite_loader::loader::AsepriteFile::load(&ase_bytes).map_err(|e| {
        darling::Error::custom(format!("Loading error for file {file_path}: {e}")).with_span(&span)
    }) {
        Ok(raw) => raw,
        Err(e) => return e.write_errors().into(),
    };

    raw.tags(); // TODO: use that

    items.push(syn::parse_quote! {
        pub const MACRO_NAME: &str = "OKOK";
    });

    quote!(#module).into()
}
