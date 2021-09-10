use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemTrait};

#[macro_use]
mod helpers;

#[macro_use]
mod expansion;

fn to_syn_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
  let compile_errors = errors.iter().map(syn::Error::to_compile_error);
  quote!(#(#compile_errors)*)
}

fn to_darling_compile_errors(errors: Vec<darling::Error>) -> proc_macro2::TokenStream {
  let compile_errors = errors.into_iter().map(|err| err.write_errors());
  quote!(#(#compile_errors)*)
}

#[proc_macro_attribute]
pub fn restcrab(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(args as AttributeArgs);

  let args_parsed = match expansion::Args::from_list(&args) {
    Ok(v) => v,
    Err(e) => {
      return TokenStream::from(e.write_errors());
    }
  };

  // into_ok_or_err()
  match expansion::on_trait(&args_parsed, &mut parse_macro_input!(input as ItemTrait)) {
    Ok(ok) => ok,
    Err(err) => err,
  }
  .into()
}
