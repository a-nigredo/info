#[macro_use]
extern crate serde_derive;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate mdo_future;
extern crate mdo;
extern crate prettytable;
extern crate clap;

use clap::{Arg, App};
use prettytable::*;
use mdo::*;
use hyper::Client;
use hyper::{Method, Request, Body};
use hyper::header::HeaderValue;
use hyper_tls::HttpsConnector;
use hyper::rt::{self, Future, Stream};
use serde::de;
use prettytable::Table;
use mdo_future::future::{bind, ret};

#[derive(Deserialize, Debug, Clone)]
struct User {
    pub login: String,
    pub repos_url: String,
    pub location: String,
    pub name: String,
    #[serde(skip_deserializing)]
    pub repos: Vec<Repo>
}

#[derive(Deserialize, Debug, Clone)]
struct Repo {
    pub name: String,
    pub full_name: String,
    pub created_at: String,
    #[serde(default, deserialize_with="deserialize_some")]
    pub description: Option<String>
}

fn main() {
    rt::run(mdo! {
        let matches = App::new("Github user repo viewer")
                    .version("1.0")
                    .author("Andrii Ivanov <a.nigredo@gmail.com>")
                    .arg(Arg::with_name("user-agent").short("u").required(true).takes_value(true))
                    .get_matches();

        let user_name = matches.value_of("user-agent").unwrap();
        let users_uri = format!("https://api.github.com/users/{}", user_name).parse().unwrap();
        let client = Client::builder().build(
            HttpsConnector::new(4).expect("TLS initialization failed")
        );
        user_response =<< client.request(mk_get_request(users_uri, user_name));
        user =<< user_response.into_body().concat2().map(|x| to::<User>(x.to_vec()).unwrap());
        repos_response =<< client.request(mk_get_request(user.repos_url.parse().unwrap(), user_name));
        repos =<< repos_response.into_body().concat2().map(|x| to::<Vec<Repo>>(x.to_vec()).unwrap());
        ret ret({
            let user = User {repos: repos, ..user};
            let mut table = Table::new();
            println!("Login: {}, Name: {}, Location: {}", user.login, user.name, user.location);
            println!("\r\nRepos: {}", user.repos.len());
            table.add_row(row!["Name", "Full name", "Description", "Created at"]);
            for v in &user.repos {
                let empty_str = &"".to_string();
                let row = row![v.name, v.full_name, v.description.as_ref().unwrap_or(empty_str), v.created_at];
                table.add_row(row);
            }
            table.printstd();
        })
    }.map_err(|err| {
        println!("Error: {}", err);
    }))
}

fn mk_get_request(uri: hyper::Uri, user_agent: &str) -> Request<hyper::Body> {
    let mut request = Request::new(Body::empty());
    *request.method_mut() = Method::GET;
    *request.uri_mut() = uri.clone();
    request.headers_mut().insert(hyper::header::ACCEPT,
                                 HeaderValue::from_static("application/vnd.github.v3+json"));
    request.headers_mut().insert(hyper::header::USER_AGENT,
                                 HeaderValue::from_str(user_agent).unwrap());
    request
}

fn to<T>(content: Vec<u8>) -> serde_json::Result<T> where for<'de> T: de::Deserialize<'de> {
    serde_json::from_slice(&content)
}

fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: de::Deserialize<'de>, D: de::Deserializer<'de> {
    de::Deserialize::deserialize(deserializer)
}
