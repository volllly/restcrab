# Restcrab
[![crates.io](https://img.shields.io/crates/v/restcrab)](https://crates.io/crates/restcrab)
[![docs.rs](https://docs.rs/restcrab/badge.svg)](https://docs.rs/restcrab/)

Restcrab provides a procedural macro [`restcrab`](crate::restcrab) and a trait [`Restcrab`](crate::Restcrab) for generating a REST client from a trait definition.

## Usage

```rust no_run
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use restcrab::{restcrab, Restcrab, crabs::reqwest};

#[derive(Serialize, Deserialize)]
struct Request {
  id: i32
}

#[derive(Serialize, Deserialize)]
struct Response {
  message: String
}

#[restcrab(crab = "reqwest::Reqwest")]
trait Service {
  #[restcrab(method = "GET", uri = "/empty")]
  fn uri_from_attribute();

  #[restcrab(method = "GET", uri = "/empty/{name}")]
  fn uri_with_parameter(#[parameter] name: &str);

  #[restcrab(method = "GET")]
  fn uri_from_method_name();

  #[restcrab(method = "GET", uri = "/static_header", header("Content-Type", "application/json"))]
  fn static_header();

  #[restcrab(method = "GET", uri = "/static_headers", header("Content-Type", "application/json"), header("User-Agen", "Restcrab"))]
  fn static_headers();

  #[restcrab(method = "GET", uri = "/static_query", query("some", "query"), query("another", "one"))]
  fn static_queries();

  #[restcrab(method = "POST", uri = "/static_body", body = "0")]
  fn static_body() -> String;

  #[restcrab(method = "POST", uri = "/dynamic_headers")]
  fn dynamic_headers(#[headers] headers: HashMap<String, String>) -> String;

  #[restcrab(method = "POST", uri = "/dynamic_queries")]
  fn dynamic_queries(#[queries] queries: HashMap<String, String>) -> String;

  #[restcrab(method = "POST", uri = "/dynamic_headers", header("Content-Type", "application/json"))]
  fn both_headers(#[headers] headers: HashMap<String, String>) -> String;

  #[restcrab(method = "POST", uri = "/dynamic_body")]
  fn dynamic_body(#[body] body: Request) -> Response;
}

fn main() {
  let client = ServiceClient::from_options(reqwest::Options {
    base_url: "https://service.url".parse().unwrap()
  }).unwrap(); 

  let mut headers = HashMap::new();
  headers.insert("User-Agent".to_string(), "Restcrab".to_string());

  let mut queries = HashMap::new();
  queries.insert("key".to_string(), "value".to_string());

  let uri_from_attribute:   ()        = client.uri_from_attribute().unwrap();
  let uri_from_method_name: ()        = client.uri_from_method_name().unwrap();
  let uri_with_parameter:   ()        = client.uri_with_parameter("value").unwrap();
  let static_header:        ()        = client.static_header().unwrap();
  let static_headers:       ()        = client.static_headers().unwrap();
  let static_queries:       ()        = client.static_queries().unwrap();
  let static_body:          String    = client.static_body().unwrap();
  let dynamic_headers:      String    = client.dynamic_headers(headers.clone()).unwrap();
  let dynamic_queries:      String    = client.dynamic_queries(queries).unwrap();
  let both_headers:         String    = client.both_headers(headers).unwrap();
  let dynamic_body:         Response  = client.dynamic_body(Request { id: 0 }).unwrap();
}
```

## Modular Backends (crabs)

Because I like to use unhelpful terminology a backend for restcrab is called a crab.

Types which implement the [`Restcrab`](crate::Restcrab) trait can be used as crabs.

The crate provides one crab which uses the [Reqwest](https://docs.rs/reqwest) http client.

If you want to implement your own crab please look at the [provided implementation](crate::crabs::reqwest) as starting off point 
