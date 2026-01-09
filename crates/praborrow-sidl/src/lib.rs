use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// automatic implementation of the `Diplomat` trait.
/// 
/// This macro calculates a stable logic hash based on the struct name and fields
/// to generate a unique `TYPE_ID`.
#[proc_macro_derive(Diplomat)]
pub fn derive_diplomat(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Calculate a simplistic stable hash
    // In a real implementation, we would hash the recursive types of fields.
    // Here we just hash the struct name to demonstrate the concept.
    let mut hasher = DefaultHasher::new();
    name.to_string().hash(&mut hasher);
    // Force non-zero
    let hash = hasher.finish() as u128;
    // ensure it's not 0 (though unlikely)
    let type_id = if hash == 0 { 1 } else { hash };

    let expanded = quote! {
        impl crate::Diplomat for #name {
            const TYPE_ID: u128 = #type_id;
        }
    };

    TokenStream::from(expanded)
}
