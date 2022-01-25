use std::str::FromStr;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

#[derive(Debug)]
struct Url(http::Uri);

impl FromMeta for Url {
  fn from_value(value: &syn::Lit) -> darling::Result<Self> {
    if let syn::Lit::Str(str) = value {
      Ok(Url(http::Uri::from_str(&str.value()).map_err(darling::Error::custom)?))
    } else {
      Err(darling::Error::custom("url needs to be a string literal"))
    }
  }
}

#[derive(Debug, Default)]
struct Method(http::Method);

impl FromMeta for Method {
  fn from_value(value: &syn::Lit) -> darling::Result<Self> {
    if let syn::Lit::Str(str) = value {
      Ok(Method(http::Method::from_str(&str.value()).map_err(darling::Error::custom)?))
    } else {
      Err(darling::Error::custom("method needs to be a string literal"))
    }
  }
}

#[derive(Debug, Default)]
struct Header(String, String);

impl FromMeta for Header {
  fn from_list(items: &[syn::NestedMeta]) -> darling::Result<Self> {
    let mut key: Option<String> = None;
    let mut value: Option<String> = None;

    if items.len() != 2 {
      return Err(darling::Error::custom("specify one key and one value"));
    }

    for (i, item) in items.iter().enumerate() {
      if let syn::NestedMeta::Lit(lit) = item {
        if let syn::Lit::Str(str) = lit {
          if i == 0 {
            key = Some(str.value())
          }
          if i == 1 {
            value = Some(str.value())
          }
        } else {
          return Err(darling::Error::custom("key and one value need to be string literals"));
        }
      } else {
        return Err(darling::Error::custom("key and one value need to be literals"));
      }
    }

    Ok(Header(key.unwrap(), value.unwrap()))
  }
}

#[derive(Debug, FromMeta)]
struct SigArgs {
  #[darling(default)]
  pub method: Method,

  #[darling(default)]
  pub uri: Option<Url>,

  #[darling(multiple, default)]
  pub header: Vec<Header>,

  #[darling(default)]
  pub body: Option<String>,
}

// #[derive(Debug, Default, FromMeta)]
// struct ParArgs {
//   #[darling(default)]
//   pub headers: bool,

//   #[darling(default)]
//   pub body: bool,
// }

pub fn on_sig(attrs: &[syn::Attribute], input: &mut syn::Signature) -> Result<syn::Block, TokenStream> {
  let mut darling_errors: Vec<darling::Error> = vec![];
  let mut syn_errors: Vec<syn::Error> = vec![];

  let sig_args = attrs
    .iter()
    .find(|a| a.path == syn::Path::from_string("restcrab").unwrap())
    .and_then(|a| Some(ok_or_push!(SigArgs::from_meta(&ok_or_push!(a.parse_meta(), syn_errors, return None)), darling_errors, return None)))
    .ok_or_else(|| darling::Error::custom(format!("Attribute not provided on fn {}", input.ident)).with_span(input).write_errors())?;

  let mut headers: Option<syn::Ident> = None;
  let mut body: Option<(syn::Type, syn::Ident)> = None;

  for parameter in &mut input.inputs {
    if let syn::FnArg::Typed(pat_type) = parameter {
      let has_header = pat_type.attrs.iter().any(|a| a.path == syn::Path::from_string("headers").unwrap());

      let has_body = pat_type.attrs.iter().any(|a| a.path == syn::Path::from_string("body").unwrap());

      pat_type.attrs = vec![];

      if has_header {
        if headers.is_none() {
          headers = if let syn::Pat::Ident(ident) = pat_type.pat.as_ref() {
            Some(ident.ident.clone())
          } else {
            darling_errors.push(darling::Error::custom(format!("Pattern {:?} is no identifier", pat_type.pat)).with_span(pat_type));
            None
          };
        } else {
          darling_errors.push(darling::Error::custom("headers attr was already set".to_string()).with_span(pat_type));
        }
      }

      if has_body {
        if body.is_none() {
          body = if let syn::Pat::Ident(ident) = pat_type.pat.as_ref() {
            Some((*pat_type.ty.clone(), ident.ident.clone()))
          } else {
            darling_errors.push(darling::Error::custom(format!("Pattern {:?} is no identifier", pat_type.pat)).with_span(pat_type));
            None
          };
        } else {
          darling_errors.push(darling::Error::custom("body attr was already set".to_string()).with_span(pat_type));
        }
      }
    }
  }

  input.inputs.insert(0, parse_quote!(&self));

  let request_type: syn::Type = if let Some(body) = &body {
    body.0.clone()
  } else if sig_args.body.is_some() {
    parse_quote! {&str}
  } else {
    parse_quote! {()}
  };
  let default_type = parse_quote!(());
  let (response_type, expect_body): (&syn::Type, bool) = match &input.output {
    syn::ReturnType::Default => (&default_type, false),
    syn::ReturnType::Type(_, return_type) => (return_type, true),
  };

  let uri_content = if let Some(uri) = sig_args.uri { uri.0.to_string() } else { format!("/{}", input.ident) };

  let method_content = {
    let method: TokenStream = sig_args
      .method
      .0
      .as_str()
      .parse()
      .map_err(|err| darling::Error::custom(format!("could not parse method: {}", err)).with_span(input).write_errors())?;
    quote! { ::restcrab::http::Method::#method }
  };

  let body_content = if let Some(body) = body {
    let ident = body.1;
    quote! {Some(#ident)}
  } else if sig_args.body.is_some() {
    let content = sig_args.body.as_ref().unwrap().to_owned();
    quote! {Some(#content)}
  } else {
    quote! {None}
  };

  let headers_content1 = if let Some(headers) = headers {
    let ident = headers;
    quote! {
      for (key, value) in #ident {
        __headers.insert(key, value);
      }
    }
  } else {
    TokenStream::new()
  };

  let headers_content2 = {
    let mut content: Vec<TokenStream> = vec![];
    for header in sig_args.header {
      let key = header.0;
      let value = header.1;
      content.push(quote! {__headers.insert(#key.to_string(), #value.to_string());});
    }
    quote! {
      #(#content)*
    }
  };

  let unwrap_response = {
    let call = quote! {
      self.call::<#request_type, #response_type>(::restcrab::Request {
        method: #method_content,
        url: #uri_content.parse::<::restcrab::http::Uri>().unwrap(),
        headers: __headers,
        body: #body_content,
        expect_body: #expect_body
      })?
    };

    if expect_body {
      quote! { Ok(#call.unwrap()) }
    } else {
      quote! { #call; Ok(()) }
    }
  };

  let block: syn::Block = parse_quote! {
    {
      let mut __headers = ::std::collections::HashMap::<String, String>::new();
      #headers_content1;
      #headers_content2;

      #unwrap_response
    }
  };

  if !darling_errors.is_empty() || !syn_errors.is_empty() {
    let darling = crate::to_darling_compile_errors(darling_errors);
    let syn = crate::to_syn_compile_errors(syn_errors);
    return Err(quote! {#darling #syn});
  }

  Ok(block)
}
