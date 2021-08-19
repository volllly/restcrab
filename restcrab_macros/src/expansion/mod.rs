use darling::FromMeta;

mod on_trait;
pub use on_trait::on_trait;

mod on_sig;
use on_sig::on_sig;

#[derive(Debug, FromMeta)]
pub struct Args {
  #[darling(default, rename = "struct")]
  pub on: Option<syn::Ident>,
  pub crab: syn::Path
}
