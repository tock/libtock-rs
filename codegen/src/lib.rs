#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;

use quote::quote;
use syn;
use syn::Error;
use syn::ItemFn;

#[allow(clippy::needless_doctest_main)]
/// Procedural attribute macro. This is meant to be applied to a binary's async
/// `main()`, transforming into a function that returns a type acceptable for
/// `main()`. In other words, this will not compile with libtock-rs:
/// ```ignore
/// async fn main() {
///     // async code
/// }
/// ```
/// and this will:
/// ```ignore
/// #[libtock::main]
/// async fn main() {
///     // async code
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    generate_main_wrapped(input.into()).into()
}

fn generate_main_wrapped(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    try_generate_main_wrapped(input).unwrap_or_else(|err| err.to_compile_error())
}

fn try_generate_main_wrapped(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, Error> {
    let ast = syn::parse2::<ItemFn>(input)?;
    let block = ast.block;
    let output = &ast.sig.output;
    Ok(quote!(
        fn main() #output {
            static mut MAIN_INVOKED: bool = false;
            unsafe {
                if MAIN_INVOKED {
                    panic!("Main called recursively; this is unsafe with #[libtock::main]");
                }
                MAIN_INVOKED = true;
            }
            let _block = async #block;
            unsafe {::core::executor::block_on(_block) }
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wraps_main_into_blocking_executor() {
        let method_def: proc_macro2::TokenStream = quote! {
            async fn main() -> ::libtock::result::TockResult<()>{
                method_call().await;
            }
        };
        let actual: ItemFn = syn::parse2::<ItemFn>(generate_main_wrapped(method_def)).unwrap();
        let expected: ItemFn = syn::parse2::<ItemFn>(quote!(
            fn main() -> ::libtock::result::TockResult<()> {
                static mut MAIN_INVOKED: bool = false;
                unsafe {
                    if MAIN_INVOKED {
                        panic!("Main called recursively; this is unsafe with #[libtock::main]");
                    }
                    MAIN_INVOKED = true;
                }
                let _block = async {
                    method_call().await;
                };
                unsafe { ::core::executor::block_on(_block) }
            }
        ))
        .unwrap();
        assert_eq!(actual, expected);
    }
}
