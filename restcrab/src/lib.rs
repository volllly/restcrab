#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "../../README.md"))]
use std::collections::HashMap;

pub use restcrab_macros::*;

#[derive(Debug, snafu::Snafu)]
pub enum Error {
  #[snafu(display("Empty response body"))]
  EmptyBody,

  #[snafu(display("Expected empty response body"))]
  NoEmptyBody,
}

pub struct Request<T> {
  pub method: http::Method,
  pub url: http::Uri,
  pub headers: HashMap<String, String>,
  pub body: Option<T>,
  pub expect_body: bool,
}

pub trait Restcrab where Self: Sized {
  type Error: std::error::Error + std::fmt::Debug + From<Error> + 'static + Send + Sync;
  type Options;
  type Crab: Restcrab;

  fn call<REQ: serde::Serialize, RES: for<'de> serde::Deserialize<'de>>(&self, request: Request<REQ>) -> Result<Option<RES>, Self::Error>;
  fn from_options(options: Self::Options) -> Result<Self, Self::Error>;
  fn options(&self) -> &Self::Options;
  fn options_mut(&mut self) -> &mut Self::Options;
}

pub mod crabs;

pub use http;
