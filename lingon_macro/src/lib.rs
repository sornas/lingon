use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};
use quote::quote;

lazy_static! {
    static ref PERF_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[proc_macro]
pub fn perf_counter(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let value = PERF_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tokens = quote! {
        #value
    };
    proc_macro::TokenStream::from(tokens)
}
