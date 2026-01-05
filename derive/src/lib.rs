use std::fs;

use heck::ToShoutySnekCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemMod, parse_macro_input};

use darling::{FromMeta, ast::NestedMeta};

#[derive(Debug, FromMeta)]
struct MacroArgs {
    /// #[aseprite(file = "...")]
    file: String,
}

#[proc_macro_attribute]
pub fn aseprite(attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let file = args.file;
    let file_path = format!("assets/{file}");
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

    items.push(syn::parse_quote! {
        pub const PATH: &'static str = #file;
    });

    let tag_names = raw
        .tags()
        .iter()
        .map(|tag| format_ident!("{}", tag.name.TO_SHOUTY_SNEK_CASE()));
    let tag_values = raw.tags().iter().map(|tag| &tag.name);
    items.push(syn::parse_quote! {
        pub mod tags {
            #( pub const #tag_names: ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new(#tag_values); )*
        }
    });

    let slice_names = raw
        .slices()
        .iter()
        .map(|slice| format_ident!("{}", slice.name.TO_SHOUTY_SNEK_CASE()));
    let slice_values = raw.slices().iter().map(|slice| &slice.name);
    items.push(syn::parse_quote! {
        pub mod slices {
            #( pub const #slice_names: ::bevy_mod_aseprite::AsepriteSlice = ::bevy_mod_aseprite::AsepriteSlice::new(#slice_values); )*
        }
    });

    quote!(#module).into()
}
