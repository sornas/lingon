use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use quote::quote;


lazy_static! {
    static ref PERF_COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

#[proc_macro]
pub fn perf_counter(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut counter = PERF_COUNTER.lock().unwrap();
    *counter = (*counter) + 1;
    let value = *counter;

    let tokens = quote! {
        #value
    };
    proc_macro::TokenStream::from(tokens)
}
