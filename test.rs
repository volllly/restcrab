#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
use restcrab::{Restcrab, crabs::reqwest::*, restcrab};
use std::{collections::HashMap, str::FromStr};
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const on_trait: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("on_trait"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::Unknown,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(on_trait())),
};
fn on_trait() {
    pub struct CrabClient {
        #[doc(hidden)]
        __restcrab: Reqwest,
    }
    impl ::restcrab::Restcrab for CrabClient {
        type Error = <Reqwest as ::restcrab::Restcrab>::Error;
        type Options = <Reqwest as ::restcrab::Restcrab>::Options;
        type Crab = Reqwest;
        fn call<REQ: ::serde::Serialize, RES: for<'de> ::serde::Deserialize<'de>>(
            &self,
            request: ::restcrab::Request<REQ>,
        ) -> Result<Option<RES>, Self::Error> {
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
        fn from_options(options: Self::Options) -> Self {
            Self {
                __restcrab: Reqwest::from_options(options),
            }
        }
        fn options(&self) -> &Self::Options {
            self.__restcrab.options()
        }
        fn options_mut(&mut self) -> &mut Self::Options {
            self.__restcrab.options_mut()
        }
    }
    impl CrabClient {
        fn from_crab(from: Reqwest) -> Self {
            Self { __restcrab: from }
        }
    }
    trait Crab: ::restcrab::Restcrab {
        fn echo(
            &self,
            body: String,
        ) -> ::std::result::Result<String, <Reqwest as ::restcrab::Restcrab>::Error>
        where
            <Reqwest as ::restcrab::Restcrab>::Error: ::std::convert::From<::restcrab::Error>,
        {
            let mut __headers = ::std::collections::HashMap::<String, String>::new();
            __headers.insert("Content-Type".to_string(), "application/json".to_string());
            Ok(self
                .call::<String, String>(::restcrab::Request {
                    method: ::restcrab::http::Method::POST,
                    url: "/echo".parse::<::restcrab::http::Uri>().unwrap(),
                    headers: __headers,
                    body: Some(body),
                    expect_body: true,
                })?
                .unwrap())
        }
        fn test(
            &self,
            headers: HashMap<String, String>,
        ) -> ::std::result::Result<String, <Reqwest as ::restcrab::Restcrab>::Error>
        where
            <Reqwest as ::restcrab::Restcrab>::Error: ::std::convert::From<::restcrab::Error>,
        {
            let mut __headers = ::std::collections::HashMap::<String, String>::new();
            for (key, value) in headers {
                __headers.insert(key, value);
            }
            __headers.insert("Content-Type".to_string(), "application/json".to_string());
            Ok(self
                .call::<&str, String>(::restcrab::Request {
                    method: ::restcrab::http::Method::POST,
                    url: "/test".parse::<::restcrab::http::Uri>().unwrap(),
                    headers: __headers,
                    body: Some("test"),
                    expect_body: true,
                })?
                .unwrap())
        }
        fn get(
            &self,
            headers: HashMap<String, String>,
        ) -> ::std::result::Result<(), <Reqwest as ::restcrab::Restcrab>::Error>
        where
            <Reqwest as ::restcrab::Restcrab>::Error: ::std::convert::From<::restcrab::Error>,
        {
            let mut __headers = ::std::collections::HashMap::<String, String>::new();
            for (key, value) in headers {
                __headers.insert(key, value);
            }
            __headers.insert("Content-Type".to_string(), "application/json".to_string());
            self.call::<(), ()>(::restcrab::Request {
                method: ::restcrab::http::Method::GET,
                url: "/get".parse::<::restcrab::http::Uri>().unwrap(),
                headers: __headers,
                body: None,
                expect_body: false,
            })?;
            Ok(())
        }
    }
    impl Crab for CrabClient {}
    CrabClient::from_options(Options {
        base_url: http::Uri::from_str("localhost").unwrap(),
    });
    CrabClient::from_crab(Reqwest::from_options(Options {
        base_url: http::Uri::from_str("localhost").unwrap(),
    }));
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&on_trait])
}
