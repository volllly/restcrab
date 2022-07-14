use std::str::FromStr;

use snafu::prelude::*;

pub struct Options {
  pub base_url: http::Uri,
}
pub struct Reqwest {
  options: Options,
  client: reqwest_lib::blocking::Client,
}

impl crate::Restcrab for Reqwest {
  type Error = Error;
  type Options = Options;
  type Crab = Reqwest;

  fn options(&self) -> &Self::Options {
    &self.options
  }

  fn options_mut(&mut self) -> &mut Self::Options {
    &mut self.options
  }

  fn from_options(options: Options) -> Result<Reqwest, Error> {
    Ok(Reqwest {
      options,
      client: reqwest_lib::blocking::Client::new(),
    })
  }

  fn call<REQ: serde::Serialize, RES: for<'de> serde::Deserialize<'de>>(&self, request: crate::Request<REQ>) -> Result<Option<RES>, Self::Error> {
    let url: http::Uri = if request.url.host().is_some() && request.url.scheme().is_some() {
      request.url.to_owned()
    } else {
      let mut base_parts = http::uri::Parts::from(self.options.base_url.clone());
      let parts = request.url.to_owned().into_parts();

      if parts.scheme.is_some() {
        base_parts.scheme = parts.scheme;
      }

      if parts.authority.is_some() {
        base_parts.authority = parts.authority;
      }

      if let Some(path_and_query) = parts.path_and_query {
        let mut path = path_and_query.path().to_string();
        if !path.starts_with('/') {
          let base_path = if let Some(path_and_query) = base_parts.path_and_query {
            path_and_query.path().to_owned()
          } else {
            "/".to_string()
          };

          if !path.ends_with('/') {
            path += "/";
          }

          path = base_path + &path;
        }

        base_parts.path_and_query = Some(http::uri::PathAndQuery::from_str(&(path + path_and_query.query().unwrap_or_default())).map_err(|source| Error::ConstructingUrl { source })?);
      }

      http::Uri::from_parts(base_parts)?
    };

    let mut req_builder = match &request.method {
      &http::Method::HEAD => self.client.head(url.to_string()),
      &http::Method::GET => self.client.get(url.to_string()),
      &http::Method::POST => self.client.post(url.to_string()),
      &http::Method::PUT => self.client.put(url.to_string()),
      &http::Method::PATCH => self.client.patch(url.to_string()),
      &http::Method::DELETE => self.client.delete(url.to_string()),
      &http::Method::OPTIONS => self.client.request(reqwest_lib::Method::OPTIONS, url.to_string()),
      &http::Method::CONNECT => self.client.request(reqwest_lib::Method::CONNECT, url.to_string()),
      &http::Method::TRACE => self.client.request(reqwest_lib::Method::TRACE, url.to_string()),
      method => return Err(Error::InvalidMethod { method: method.clone() }),
    };

    for (key, value) in &request.headers {
      req_builder = req_builder.header(key, value);
    }

    req_builder = req_builder.query(&request.queries.iter().collect::<Vec<_>>());

    if let Some(body) = &request.body {
      req_builder = req_builder.body(serde_json::to_string(body).context(SerializingBodySnafu)?);
    }

    let response = req_builder.send().context(SendingRequestSnafu)?;

    ensure!(response.status().is_success(), UnsuccessfulResponseCodeSnafu { response });

    let text = response.text().context(DecodingResponseBodySnafu)?;

    if !text.is_empty() {
      serde_json::from_str::<RES>(text.as_str()).map(Some).context(DeserializingBodySnafu)
    } else {
      Ok(None)
    }
  }
}

#[derive(Debug, snafu::Snafu)]
pub enum Error {
  #[snafu(display("Error parsing url: {source}"), context(false))]
  ParsingUrl { source: http::uri::InvalidUriParts },

  #[snafu(display("Error serializing body: {source}"))]
  SerializingBody { source: serde_json::Error },

  #[snafu(display("Error deserializing body: {source}"))]
  DeserializingBody { source: serde_json::Error },

  #[snafu(display("Error sending request: {source}"))]
  SendingRequest { source: reqwest_lib::Error },

  #[snafu(display("Unsuccessful response code: {response:?}"))]
  UnsuccessfulResponseCode { response: reqwest_lib::blocking::Response },

  #[snafu(display("Error converting response body to text: {source}"))]
  DecodingResponseBody { source: reqwest_lib::Error },

  #[snafu(display("Invalid method: {method}"))]
  InvalidMethod { method: http::Method },

  #[snafu(display("Error constructing url: {source}"))]
  ConstructingUrl { source: http::uri::InvalidUri },

  #[snafu(context(false))]
  Restcrab { source: crate::Error },
}
