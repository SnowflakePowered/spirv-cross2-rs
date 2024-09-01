use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Type};

#[derive(Debug)]
struct CompilerOption {
    field_name: Ident,
    path: TokenStream,
    is_bool: bool,
}

pub(crate) fn do_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new(
            input.span(),
            "CompilerOptions can only be derived for structs.",
        ));
    };

    // Special-case empty struct, otherwise we don't support tuple structs.
    if data.fields.is_empty() {
        let name = input.ident;
        // Build the output, possibly using quasi-quotation
        let expanded = quote! {
        impl CompilerOptions for #name {
            unsafe fn apply<'a>(&self, _options: ::spirv_cross_sys::spvc_compiler_options, _root: impl ContextRooted + Copy)
                -> crate::error::Result<()>
                { Ok(()) }
            }
        };
        return Ok(expanded)
    }

    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "CompilerOptions can not be derived for tuple structs.",
        ));
    };

    let options: Vec<_> = fields
        .named
        .iter()
        .filter_map(|field| {
            let ident = field.ident.clone().unwrap();
            let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("option")) else {
                return None;
            };

            let Ok(name) = attr.meta.require_list() else {
                return None;
            };

            let is_bool = match &field.ty {
                Type::Path(type_path) if type_path.path.is_ident("bool") => true,
                _ => false,
            };

            Some(CompilerOption {
                field_name: ident,
                path: name.tokens.clone(),
                is_bool,
            })
        })
        .collect();

    let mut setters = Vec::new();
    for option in options {
        let path = option.path;
        let field = option.field_name;

        let setter = if option.is_bool {
            quote! {
                ::spirv_cross_sys::spvc_compiler_options_set_bool(options, #path, self.#field)
                .ok(root)?;

            }
        } else {
            quote! {
                ::spirv_cross_sys::spvc_compiler_options_set_uint(options, #path,
                    ::std::os::raw::c_uint::from(self.#field))
                .ok(root)?;
            }
        };

        setters.push(setter);
    }

    let name = input.ident;
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl CompilerOptions for #name {
            unsafe fn apply<'a>(&self, options: ::spirv_cross_sys::spvc_compiler_options, root: impl ContextRooted + Copy)
                -> crate::error::Result<()>
            {
                unsafe {
                    #(#setters)*;
                }

                Ok(())
            }
        }
    };

    Ok(expanded)
}
