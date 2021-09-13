use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_quote;

pub fn on_trait(args: &super::Args, input: &mut syn::ItemTrait) -> Result<TokenStream, TokenStream> {
  let errors: Vec<syn::Error> = vec![];
  let mut error_tokens = TokenStream::new();
  let mut original_trait = input.clone();

  input.ident = format_ident!("{}Crab", input.ident);
  let trait_name = &input.ident;
  let struct_name = args.on.clone().unwrap_or_else(|| format_ident!("{}Client", original_trait.ident));
  let crab_name = &args.crab;
  let empty_vec: Vec<syn::Meta> = vec![];
  let crab_trait_attributes: Vec<TokenStream> = args.attributes
    .as_ref()
    .map(|attrs| &attrs.crab)
    .unwrap_or(&empty_vec)
    .iter()
    .filter_map(|a| {
      if let syn::Meta::List(list) = a {
        if list.nested.len() != 1 {
          None
        } else {
          Some(&list.nested[0])
        }
      } else {
        None
      }
    })
    .map(|n| quote!(#[#n])).collect();

  let client_trait_attributes: Vec<TokenStream> = args.attributes
    .as_ref()
    .map(|attrs| &attrs.client)
    .unwrap_or(&empty_vec)
    .iter()
    .filter_map(|a| {
      if let syn::Meta::List(list) = a {
        if list.nested.len() != 1 {
          None
        } else {
          Some(&list.nested[0])
        }
      } else {
        None
      }
    })
    .map(|n| quote!(#[#n])).collect();

  for item in &mut input.items {
    if let syn::TraitItem::Method(method) = item {
      let expanded = match super::on_sig(&method.attrs, &mut method.sig) {
        Ok(expanded) => expanded,
        Err(err) => {
          error_tokens = quote! {#error_tokens #err};
          continue;
        }
      };
      let default_type = parse_quote!(());
      let output: &syn::Type = match &method.sig.output {
        syn::ReturnType::Default => &default_type,
        syn::ReturnType::Type(_, return_type) => return_type,
      };
      method.sig.output = parse_quote!(-> ::std::result::Result<#output, <#crab_name as ::restcrab::Restcrab>::Error>);
      method.sig.generics.where_clause = parse_quote! {
        where
          <#crab_name as ::restcrab::Restcrab>::Error: ::std::convert::From<<Self as ::restcrab::Restcrab>::Error>
      };

      method.attrs = vec![];

      method.default = Some(expanded);
    }
  }

  input.supertraits = parse_quote!(::restcrab::Restcrab);
  input.attrs = vec![];

  if !errors.is_empty() {
    return Err(crate::to_syn_compile_errors(errors));
  }

  for item in &mut original_trait.items {
    if let syn::TraitItem::Method(method) = item {
      method.attrs.retain(|a| a.path != syn::Path::from_string("restcrab").unwrap());
      for parameter in &mut method.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = parameter {
          pat_type.attrs.retain(|a| a.path != syn::Path::from_string("body").unwrap() && a.path != syn::Path::from_string("headers").unwrap());
        }
      }

    }
  }

  Ok(quote! {
    #original_trait

    #(#client_trait_attributes)*
    pub struct #struct_name {
      #[doc(hidden)]
      __restcrab: #crab_name
    }

    impl ::restcrab::Restcrab for #struct_name {
      type Error = <#crab_name as ::restcrab::Restcrab>::Error;
      type Options = <#crab_name as ::restcrab::Restcrab>::Options;
      type Crab = #crab_name;

      fn call<REQ: ::serde::Serialize, RES: for<'de> ::serde::Deserialize<'de>>(&self, request: ::restcrab::Request<REQ>) -> Result<Option<RES>, Self::Error> {
        let expect_body = request.expect_body;

        let response = self.__restcrab.call(request)?;
        if expect_body {
          if response.is_none() {
            Err(::restcrab::Error::EmptyBody)?;
          }
        } else if response.is_some() {
          Err(::restcrab::Error::NoEmptyBody)?;
        }

        Ok(response)
      }

      fn from_options(options: Self::Options) -> Result<Self, Self::Error> {
        Ok(Self {
          __restcrab: #crab_name::from_options(options)?
        })
      }

      fn options(&self) -> &Self::Options {
        self.__restcrab.options()
      }

      fn options_mut(&mut self) -> &mut Self::Options {
        self.__restcrab.options_mut()
      }
    }

    impl #struct_name {
      fn from_crab(from: #crab_name) -> Self {
        Self {
          __restcrab: from
        }
      }
    }

    #(#crab_trait_attributes)*
    #input

    impl #trait_name for #struct_name {}
  })
}
