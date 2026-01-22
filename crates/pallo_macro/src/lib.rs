extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[proc_macro]
pub fn property_id(input: TokenStream) -> TokenStream {
    let input_literal = parse_macro_input!(input as LitStr);

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);

    let expanded = quote! {
        PropertyId::new(#id, #input_literal)
    };

    TokenStream::from(expanded)
}
