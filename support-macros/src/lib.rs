#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;

use quote::quote;
use syn;
use syn::Error;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn libtock_main(_: TokenStream, input: TokenStream) -> TokenStream {
    generate_main_wrapped(input.into()).into()
}

fn generate_main_wrapped(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    try_generate_main_wrapped(input).unwrap_or_else(|err| err.to_compile_error().into())
}

fn try_generate_main_wrapped(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, Error> {
    let ast = syn::parse2::<ItemFn>(input)?;
    let block = ast.block;
    Ok(quote!(
        fn main() -> ::libtock::result::TockResult<()> {
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
        }
        .into();
        let actual: ItemFn = syn::parse2::<ItemFn>(generate_main_wrapped(method_def.into()))
            .unwrap()
            .into();
        let expected: ItemFn = syn::parse2::<ItemFn>(quote!(
            fn main() -> ::libtock::result::TockResult<()> {
                let _block = async {
                    method_call().await;
                };
                unsafe { ::core::executor::block_on(_block) }
            }
        ))
        .unwrap()
        .into();
        assert_eq!(actual, expected);
    }
}
