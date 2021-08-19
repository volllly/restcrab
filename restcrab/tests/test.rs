use std::collections::HashMap;
use std::convert::TryInto;

use wiremock::*;
use wiremock::matchers::*;
use restcrab::{Restcrab, crabs::reqwest::*};
use fake::{Fake, Faker};

pub struct Responder<C: Fn(&Request) -> ResponseTemplate> {
  responder: C
}

impl<C: Fn(&Request) -> ResponseTemplate> Responder<C> {
  fn from(from: C) -> Responder<C> {
    Responder {
      responder: from
    }
  }
}

impl<C: Fn(&Request) -> ResponseTemplate + Send + Sync> Respond for Responder<C> {
    fn respond(&self, request: &Request) -> ResponseTemplate {
      (self.responder)(request)
    }
}

async fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
      .and(path("/echo"))
      .and(header("Content-Type", "application/json"))
      .respond_with(Responder::from(|r| ResponseTemplate::new(200).set_body_string(String::from_utf8(r.body.to_owned()).unwrap()))).mount(&mock_server).await;

    Mock::given(method("POST"))
      .and(path("/test"))
      .and(header("Content-Type", "application/json"))
      .and(header("test", "header"))
      .and(body_string("\"test\""))
      .respond_with(ResponseTemplate::new(200).set_body_string("\"testbody\"")).mount(&mock_server).await;

    Mock::given(method("GET"))
      .and(path("/get"))
      .and(header("Content-Type", "application/json"))
      .and(header("test", "header"))
      .respond_with(ResponseTemplate::new(200)).mount(&mock_server).await;

    mock_server
}

#[restcrab::restcrab(crab = "Reqwest")]
trait Crab {
  #[restcrab(method = "POST", uri = "/echo", header("Content-Type", "application/json"))]
  fn echo(#[restcrab(body)] body: String) -> String;

  #[restcrab(method = "POST", uri = "/test", body = "test", header("Content-Type", "application/json"))]
  fn test(#[restcrab(headers)] headers: HashMap<String, String>) -> String;

  #[restcrab(method = "GET", uri = "/get", header("Content-Type", "application/json"))]
  fn get(#[restcrab(headers)] headers: HashMap<String, String>);
}

#[async_std::test]
async fn reqwest_crab() {
  let mock_server = setup_mock_server().await;
  let client = CrabClient::from_options(Options { base_url: mock_server.uri().try_into().unwrap() });

  let message: String = Faker.fake();
  let response = client.echo(message.clone()).unwrap();
  assert_eq!(message, response);

  let mut headers = HashMap::new();
  headers.insert("test".to_string(), "header".to_string());
  let response = client.test(headers).unwrap();
  assert_eq!("testbody", response);

  let mut headers = HashMap::new();
  headers.insert("test".to_string(), "header".to_string());
  client.get(headers).unwrap();
}
