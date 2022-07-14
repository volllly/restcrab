use std::{collections::HashMap, str::FromStr};

use restcrab::{crabs::reqwest::*, restcrab, Restcrab};

#[test]
fn on_trait() {
  #[restcrab(crab = "Reqwest")]
  trait Crab {
    #[restcrab(method = "POST", uri = "/echo", header("Content-Type", "application/json"), header("Content-Type", "application/json"))]
    fn echo(#[body] body: String) -> String;

    #[restcrab(method = "POST", uri = "/test", body = "test", header("Content-Type", "application/json"))]
    fn test(#[headers] headers: HashMap<String, String>) -> String;

    #[restcrab(method = "GET", uri = "/get", header("Content-Type", "application/json"))]
    fn get(#[headers] headers: HashMap<String, String>);

    #[restcrab(method = "PUT", uri = "/put", query("test", "value"), query("test2", "value2"))]
    fn static_query();

    #[restcrab(method = "DELETE", uri = "/delete")]
    fn dynamic_query(#[queries] queries: HashMap<String, String>);
  }

  CrabClient::from_options(Options {
    base_url: http::Uri::from_str("localhost").unwrap(),
  })
  .unwrap();

  CrabClient::from_crab(
    Reqwest::from_options(Options {
      base_url: http::Uri::from_str("localhost").unwrap(),
    })
    .unwrap(),
  );
}
