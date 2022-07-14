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

/// The [`restcrab`](macro@crate::restcrab) attribute macro can be used on traits.
///
/// It generates a trait `<TaritName>Crab` for the original trait and a struct `<TraitName>Client` which implements the trait `<TaritName>Crab`.
///
/// The [`restcrab`](macro@crate::restcrab) attribute macro takes a parameter `crab` which defined the backend to use and an optional parameter
/// `attributes` which contains attributes to add to the generated trait and struct.
///
/// ## Add attributes
///
/// Attributes can be added to the generated trait like this.
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// #[restcrab(crab = "Reqwest", attributes(crab(cfg(not(feature = "some-feature")))))]
/// trait Service {}
/// ```
/// This adds the attribute `#[cfg(not(feature = "some-feature"))]` to the generated trait.
///
/// Attributes can be added to the generated struct like this.
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// #[restcrab(
///   crab = "Reqwest",
///   attributes(client(cfg(not(feature = "some-feature"))))
/// )]
/// trait Service {}
/// ```
/// This adds the attribute `#[cfg(not(feature = "some-feature"))]` to the generated struct.
///
/// ## Select http method
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET")]
///   fn method();
/// }
/// ```
///
/// ## Select uri
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", uri = "/url/path/to/call")]
///   fn method();
/// }
/// ```
/// Without the `uri` parameter the method name is used as `uri`.
///
/// ## Add parameters to request url
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct Request {}
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", uri = "/url/{name}")]
///   fn method(#[parameter] name: &str);
/// }
/// ```
///
/// ## Add return type
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Response {}
///
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET")]
///   fn method() -> Response;
/// }
/// ```
///
/// ## Add static headers to request
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", header("key", "value"), header("key2", "value2"))]
///   fn method();
/// }
/// ```
/// The `header` field can be added multiple times to the attribute.
///
/// ## Add static query parameters to request
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct Request {}
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", query("key", "value"))]
///   fn method();
/// }
/// ```
///
/// ## Add static body to request
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", body = "body")]
///   fn method();
/// }
/// ```
///
/// ## Add dynamic headers to request
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// # use std::collections::HashMap;
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", header("key", "value"))]
///   fn method(#[headers] headers: HashMap<String, String>);
/// }
/// ```
/// Can be combined with static headers.
///
/// ## Add dynamic query parameters to request
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// # use std::collections::HashMap;
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET", query("key", "value"))]
///   fn method(#[queries] headers: HashMap<String, String>);
/// }
/// ```
/// Can be combined with static query parameters.
///
/// ## Add dynamic body to request
/// ```
/// # use restcrab::{restcrab, crabs::reqwest::Reqwest};
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct Request {}
/// #[restcrab(crab = "Reqwest")]
/// trait Service {
///   #[restcrab(method = "GET")]
///   fn method(#[body] body: Request);
/// }
/// ```

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
