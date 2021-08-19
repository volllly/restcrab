use std::collections::HashMap;

pub use restcrab_macros::*;

#[derive(Debug, thiserror::Error)]
pub enum Error<T: InnerError> {
  #[error("{0}")]
  Crab(#[from] T),
  #[error("{0}")]
  Http(#[from] http::Error),
  #[error("Empty response body")]
  EmptyBody,
  #[error("Expected empty response body")]
  NoEmptyBody
}

pub trait InnerError: std::error::Error + std::fmt::Debug + 'static {}

impl<T: InnerError> InnerError for Error<T> {}

pub struct Request<T> {
  pub method: http::Method,
  pub url: http::Uri,
  pub headers: HashMap<String, String>,
  pub body: Option<T>,
  pub expect_body: bool
}

pub trait Restcrab {
  type Error: InnerError;
  type Options;
  type Crab: Restcrab;
  
  fn call<REQ: serde::Serialize, RES: for<'de> serde::Deserialize<'de>>(&self, request: Request<REQ>) -> Result<Option<RES>, Self::Error>;
  fn from_options(options: Self::Options) -> Self;
  fn options(&self) -> &Self::Options;
  fn options_mut(&mut self) -> &mut Self::Options;
}

pub mod crabs;
