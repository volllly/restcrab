use darling::FromMeta;

mod on_trait;
pub use on_trait::on_trait;

mod on_sig;
use on_sig::on_sig;

#[derive(Debug, FromMeta)]
pub struct AttributesArgs {
  #[darling(multiple)]
  pub crab: Vec<syn::Meta>,
  #[darling(multiple)]
  pub client: Vec<syn::Meta>
}

#[derive(Debug, FromMeta)]
pub struct Args {
  #[darling(default, rename = "struct")]
  pub on: Option<syn::Ident>,
  pub crab: syn::Path,
  #[darling(default)]
  pub attributes: Option<AttributesArgs>
}
