/*
Copyright 2023 Benjamin Richcreek

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
//! # Enum Unwrapper
//!`enum_unwrapper` is a lightweight procedural macro for "unwrapping" [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>)s into their inner types when the variant is known through automatic implementation of [`TryFrom`].
//!
//!`enum_unrapper` does this by allowing the user to add a procedural macro attribute, [`macro@unique_try_froms`] to [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>) definitions.
//!
//!For more information and examples, check the attribute's [documentation](macro@unique_try_froms).
use syn;
use quote::quote;
use proc_macro::TokenStream;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
/// # Unique TryFroms
/// Add this attribute to [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>) definitions, and it will implement [`TryFrom`] for each standalone type contained in a variant of that [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>)/
/// # Example
/// ```no_run
/// #[unique_try_froms()]
/// enum NumberHolder {
///    U8(u8),
///    U16(u16),
///}
///fn main() {
///    let small_number = NumberHolder::U8(4);
///    assert_eq!(4,u8::try_from(small_number).unwrap());
///    let big_number = NumberHolder::U16(444);
///    assert_eq!(444,u16::try_from(big_number).unwrap());
///}
///```
///note: this example is not automatically tested due to restrictions on `proc_macro` crates
/// # Panics
/// The macro panics when attached to anything other than an [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>) definition.
///
/// The macro panics if attached to an [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>) with one or more variants containing multiple fields, such as
/// ```no_run
///    Variant(u8,u8),
///```
/// In such cases it is recomended to condense the data into one type, like so:
///```no_run
/// Variant([u8;2]),
///```
///
///The macro currently panics if attached to an [`enum`](<https://doc.rust-lang.org/1.58.1/std/keyword.enum.html>) definition with variants containing identical types.
#[proc_macro_attribute]
pub fn unique_try_froms (_exempt_types: TokenStream, user_enum: TokenStream) -> TokenStream {
    let parsed_enum: &syn::ItemEnum  = &syn::parse(user_enum).expect("This attribute should only be attached to a enum definition");
    let enum_name = &parsed_enum.ident;
    let ident_extractor = |variant: &syn::Variant| -> syn::Ident {
        variant.ident.clone()
    };
    let inner_type_extractor = |variant: &syn::Variant| -> syn::Type {
        match &variant.fields {
            syn::Fields::Unnamed(wrapped) => return wrapped.unnamed.first().expect("Each enum variant should contain one inner value").ty.clone(),
            _ => panic!("An unexpected error occoured, please only use unnamed enum variants")
        }
    };
    let enum_variants = parsed_enum.variants.iter().map(ident_extractor);
    let variant_types = parsed_enum.variants.iter().map(inner_type_extractor);
    quote! {
        #parsed_enum
        #(impl TryFrom<#enum_name> for #variant_types {
            type Error = &'static str;
            fn try_from(value: #enum_name) ->  Result<Self,Self::Error> {
                match value {
                    #enum_name::#enum_variants(inner) => return Ok(inner),
                    _ => return Err("Only variants containing an inner value of the same type as the target should be passed to this function"),
                }
            }
        })*
    }.into()
}
