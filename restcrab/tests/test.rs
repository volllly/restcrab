use std::{collections::HashMap, convert::TryInto};

use fake::{Fake, Faker};
use restcrab::{crabs::reqwest::*, restcrab, Restcrab};
use wiremock::{matchers::*, *};

pub struct Responder<C: Fn(&Request) -> ResponseTemplate> {
  responder: C,
}

impl<C: Fn(&Request) -> ResponseTemplate> Responder<C> {
  fn from(from: C) -> Responder<C> {
    Responder { responder: from }
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
    .respond_with(Responder::from(|r| ResponseTemplate::new(200).set_body_string(String::from_utf8(r.body.to_owned()).unwrap())))
    .mount(&mock_server)
    .await;

  Mock::given(method("POST"))
    .and(path("/test"))
    .and(header("Content-Type", "application/json"))
    .and(header("test", "header"))
    .and(body_string("\"test\""))
    .respond_with(ResponseTemplate::new(200).set_body_string("\"testbody\""))
    .mount(&mock_server)
    .await;

  Mock::given(method("GET"))
    .and(path("/get"))
    .and(header("Content-Type", "application/json"))
    .and(header("test", "header"))
    .respond_with(ResponseTemplate::new(200))
    .mount(&mock_server)
    .await;

  Mock::given(method("PUT"))
    .and(path("/put"))
    .and(query_param("test", "value"))
    .and(query_param("test2", "value2"))
    .respond_with(ResponseTemplate::new(200))
    .mount(&mock_server)
    .await;

  Mock::given(method("DELETE"))
    .and(path("/delete"))
    .and(query_param("test", "value"))
    .and(query_param("test3", "value3"))
    .and(query_param("test4", "value4"))
    .respond_with(ResponseTemplate::new(200))
    .mount(&mock_server)
    .await;

  Mock::given(method("GET"))
    .and(path("/parameter"))
    .respond_with(ResponseTemplate::new(200))
    .mount(&mock_server)
    .await;

  mock_server
}

#[restcrab::restcrab(crab = "Reqwest")]
trait Crab {
  #[restcrab(method = "POST", uri = "/echo", header("Content-Type", "application/json"))]
  fn echo(#[body] body: String) -> String;

  #[restcrab(method = "POST", uri = "/test", body = "test", header("Content-Type", "application/json"))]
  fn test(#[headers] headers: HashMap<String, String>) -> String;

  #[restcrab(method = "GET", header("Content-Type", "application/json"))]
  fn get(#[headers] headers: HashMap<String, String>);
  
  #[restcrab(method = "PUT", uri = "/put", query("test", "value"), query("test2", "value2"))]
  fn static_query();

  #[restcrab(method = "DELETE", uri = "/delete", query("test", "value"))]
  fn dynamic_query(#[queries] headers: HashMap<String, String>);
  
  #[restcrab(method = "GET", uri = "/{test}")]
  fn path_parameters(#[parameter] test: &str);
}

#[async_std::test]
async fn reqwest_crab() {
  let mock_server = setup_mock_server().await;
  let client = CrabClient::from_options(Options {
    base_url: mock_server.uri().try_into().unwrap(),
  })
  .unwrap();

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

  client.static_query().unwrap();

  let mut queries = HashMap::new();
  queries.insert("test".to_string(), "value".to_string());
  queries.insert("test3".to_string(), "value3".to_string());
  queries.insert("test4".to_string(), "value4".to_string());
  client.dynamic_query(queries).unwrap();
  
  client.path_parameters("parameter").unwrap();
}

#[restcrab(crab = "Reqwest")]
trait WrongCrab {
  #[restcrab(method = "POST", uri = "/echo", header("Content-Type", "application/json"))]
  fn echo(#[body] body: String);

  #[restcrab(method = "GET", uri = "/get", header("Content-Type", "application/json"))]
  fn get(#[headers] headers: HashMap<String, String>) -> String;
}
#[async_std::test]
async fn error_messages() {
  let mock_server = setup_mock_server().await;
  let client = WrongCrabClient::from_options(Options {
    base_url: mock_server.uri().try_into().unwrap(),
  })
  .unwrap();

  let message: String = Faker.fake();
  let response = client.echo(message);
  println!("{}", response.err().unwrap());

  let mut headers = HashMap::new();
  headers.insert("test".to_string(), "header".to_string());
  let response = client.get(headers);
  println!("{}", response.err().unwrap());
}
