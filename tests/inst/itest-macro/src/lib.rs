extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

/// Wraps function using `procspawn` to allocate a new temporary directory,
/// make it the process' working directory, and run the function.
#[proc_macro_attribute]
pub fn itest(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = syn::parse_macro_input!(attrs as syn::AttributeArgs);
    let func = syn::parse_macro_input!(input as syn::ItemFn);
    let fident = func.sig.ident.clone();
    let varident = quote::format_ident!("TEST_{}", fident);
    let fidentstrbuf = format!(r#""{}"#, fident);
    let fidentstr = syn::LitStr::new(&fidentstrbuf, Span::call_site());
    let output = quote! {
        #[distributed_slice(TESTS)]
        static #varident : Test = Test {
            name: #fidentstr,
            f: #fident,
            #(#attrs)*
        };
        #func
    };
    output.into()
}
