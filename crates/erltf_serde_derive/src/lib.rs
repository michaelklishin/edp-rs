// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Meta, parse_macro_input};

/// Derive macro for Elixir struct serialization.
///
/// This macro generates `Serialize` and `Deserialize` implementations that
/// produce Elixir-compatible struct format with:
/// - A `__struct__` field containing the module name as an atom
/// - All field keys as atoms (not binaries)
///
/// # Example
///
/// ```ignore
/// use erltf_serde::ElixirStruct;
///
/// #[derive(ElixirStruct)]
/// #[elixir_module = "MyApp.User"]
/// struct User {
///     name: String,
///     age: i32,
/// }
/// ```
///
/// This will serialize to an Erlang map equivalent to:
/// ```elixir
/// %MyApp.User{name: "...", age: ...}
/// ```
///
/// The `Elixir.` prefix is automatically added to the module name.
#[proc_macro_derive(ElixirStruct, attributes(elixir_module))]
pub fn derive_elixir_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let module_name = extract_module_name(&input)
        .unwrap_or_else(|| panic!("ElixirStruct requires #[elixir_module = \"...\"] attribute"));

    let full_module_name = format!("Elixir.{}", module_name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ElixirStruct only supports structs with named fields"),
        },
        _ => panic!("ElixirStruct can only be derived for structs"),
    };

    let field_names: Vec<&Ident> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let field_name_strs: Vec<String> = field_names.iter().map(|f| f.to_string()).collect();
    let field_count = field_names.len() + 1; // +1 for __struct__

    let serialize_impl = generate_serialize_impl(
        name,
        &impl_generics,
        &ty_generics,
        where_clause,
        &full_module_name,
        &field_names,
        &field_name_strs,
        field_count,
    );

    let deserialize_impl = generate_deserialize_impl(
        name,
        &impl_generics,
        &ty_generics,
        where_clause,
        &full_module_name,
        &field_names,
        &field_name_strs,
    );

    let expanded = quote! {
        #serialize_impl
        #deserialize_impl
    };

    TokenStream::from(expanded)
}

fn extract_module_name(input: &DeriveInput) -> Option<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("elixir_module")
            && let Meta::NameValue(meta) = &attr.meta
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = &meta.value
        {
            return Some(lit_str.value());
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn generate_serialize_impl(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    full_module_name: &str,
    field_names: &[&Ident],
    field_name_strs: &[String],
    field_count: usize,
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics serde::Serialize for #name #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeMap;

                let mut map = serializer.serialize_map(Some(#field_count))?;

                map.serialize_entry(
                    &erltf_serde::elixir::AtomKey("__struct__"),
                    &erltf_serde::elixir::AtomValue(#full_module_name),
                )?;

                #(
                    map.serialize_entry(
                        &erltf_serde::elixir::AtomKey(#field_name_strs),
                        &self.#field_names,
                    )?;
                )*

                map.end()
            }
        }
    }
}

fn generate_deserialize_impl(
    name: &Ident,
    _impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    full_module_name: &str,
    field_names: &[&Ident],
    field_name_strs: &[String],
) -> proc_macro2::TokenStream {
    let field_count = field_names.len();

    let field_declarations = field_names.iter().map(|f| {
        quote! { let mut #f = None; }
    });

    let field_assignments =
        field_name_strs
            .iter()
            .zip(field_names.iter())
            .map(|(name_str, field)| {
                quote! {
                    #name_str => {
                        #field = Some(map.next_value()?);
                    }
                }
            });

    let field_unwraps = field_names
        .iter()
        .zip(field_name_strs.iter())
        .map(|(field, name_str)| {
            quote! {
                #field: #field.ok_or_else(|| serde::de::Error::missing_field(#name_str))?
            }
        });

    let expecting_msg = format!("struct {} with {} fields", name, field_count);
    let visitor_name = Ident::new(&format!("{}Visitor", name), name.span());

    quote! {
        impl<'de> serde::Deserialize<'de> for #name #ty_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct #visitor_name;

                impl<'de> serde::de::Visitor<'de> for #visitor_name {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str(#expecting_msg)
                    }

                    fn visit_map<M>(self, mut map: M) -> core::result::Result<Self::Value, M::Error>
                    where
                        M: serde::de::MapAccess<'de>,
                    {
                        #(#field_declarations)*

                        while let Some(key) = map.next_key::<std::borrow::Cow<'de, str>>()? {
                            match key.as_ref() {
                                "__struct__" => {
                                    let module: std::borrow::Cow<'de, str> = map.next_value()?;
                                    if module.as_ref() != #full_module_name {
                                        return Err(serde::de::Error::custom(
                                            format!("expected __struct__ to be {}, got {}", #full_module_name, module)
                                        ));
                                    }
                                }
                                #(#field_assignments)*
                                _ => {
                                    let _: serde::de::IgnoredAny = map.next_value()?;
                                }
                            }
                        }

                        Ok(#name {
                            #(#field_unwraps),*
                        })
                    }
                }

                deserializer.deserialize_map(#visitor_name)
            }
        }
    }
}
