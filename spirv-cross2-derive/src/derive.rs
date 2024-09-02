use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Fields, Token, Type};

struct CompilerOption {
    field_name: Ident,
    path: Expr,
    default: Option<Expr>,
    is_bool: bool,
}

struct Expansions {
    field_name: Ident,
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
            impl crate::compile::CompilerOptions for #name { }

            impl crate::compile::sealed::ApplyCompilerOptions for #name {
                unsafe fn apply<'a>(&self, _options: ::spirv_cross_sys::spvc_compiler_options, _root: impl ContextRooted + Copy)
                    -> crate::error::Result<()>
                { Ok(()) }
            }
        };

        return Ok(expanded);
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

            let Ok(list) = attr.meta.require_list() else {
                return None;
            };

            let is_bool = match &field.ty {
                Type::Path(type_path) if type_path.path.is_ident("bool") => true,
                _ => false,
            };

            let punctuated = list
                .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                .ok()?;

            let path = punctuated.get(0)?;

            let default = punctuated.get(1).cloned();

            Some(CompilerOption {
                field_name: ident,
                path: path.clone(),
                default,
                is_bool,
            })
        })
        .collect();

    let expands: Vec<_> = fields
        .named
        .iter()
        .filter_map(|field| {
            let ident = field.ident.clone().unwrap();
            let Some(_attr) = field.attrs.iter().find(|a| a.path().is_ident("expand")) else {
                return None;
            };

            Some(Expansions { field_name: ident })
        })
        .collect();

    let mut setters = Vec::new();
    let mut defaults: Vec<TokenStream> = Vec::new();
    let mut expanders: Vec<TokenStream> = Vec::new();

    for option in options {
        let path = option.path;
        let field = option.field_name;
        let default = option.default;

        let setter = if option.is_bool {
            quote! {
                ::spirv_cross_sys::spvc_compiler_options_set_bool(options, ::spirv_cross_sys::spvc_compiler_option::#path, self.#field)
                .ok(root)?;
            }
        } else {
            quote! {
                ::spirv_cross_sys::spvc_compiler_options_set_uint(options, ::spirv_cross_sys::spvc_compiler_option::#path,
                    u32::from(self.#field))
                .ok(root)?;
            }
        };

        let default_setter = if let Some(default) = default {
            quote! {
                #field: #default,
            }
        } else {
            quote! {
                #field: Default::default(),
            }
        };

        setters.push(setter);
        defaults.push(default_setter);
    }

    for expands in expands {
        let field = expands.field_name;
        let expander = quote! {
            crate::compile::sealed::ApplyCompilerOptions::apply(&self.#field, options, root)?;
        };
        let default_setter = quote! {
             #field: Default::default(),
        };

        expanders.push(expander);
        defaults.push(default_setter);
    }

    let name = input.ident;
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl crate::compile::sealed::ApplyCompilerOptions for #name {
            unsafe fn apply<'a>(&self, options: ::spirv_cross_sys::spvc_compiler_options, root: impl ContextRooted + Copy)
                -> crate::error::Result<()>
            {
                unsafe {
                    #(#expanders)*;
                }

                unsafe {
                    #(#setters)*;
                }

                Ok(())
            }
        }

         impl ::std::default::Default for #name {
            fn default() -> Self {
                Self {
                     #(#defaults)*
                }
            }
        }

        impl crate::compile::CompilerOptions for #name { }
    };

    Ok(expanded)
}
