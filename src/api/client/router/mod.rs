//! Client API router
use super::*;

// std
use std::net::ToSocketAddrs;

// future
use futures::future::Future;

// gotham
extern crate gotham;
use gotham::bind_server;
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::middleware::cookie::CookieParser;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};

// hyper
extern crate hyper;
use hyper::header::SET_COOKIE;
use hyper::{Body, Response, StatusCode};

// failure
use failure::{err_msg, Error};

// openssl
extern crate openssl;
use openssl::{
    pkey::PKey,
    ssl::{SslAcceptor, SslMethod},
    x509::X509,
};

// tokio
use tokio::{net::TcpListener, runtime::Runtime};

// tokio_openssl
extern crate tokio_openssl;
use tokio_openssl::SslAcceptorExt;

// mime
extern crate mime;

// cookie
extern crate cookie;
use cookie::{Cookie, CookieJar, SameSite};

// serde
use serde::Serialize;

// handlers
mod handlers;

// handlers constants
const COOKIE_USER: &str = "user";
const COOKIE_TIMESTAMP: &str = "ts";
const COOKIE_NONCE: &str = "nonce";
const COOKIE_TOKEN: &str = "token";

// Client API routing
// ------------------
// c-api/
//  |_ v1/
//     |_ / GET, HEAD     index
//     |_ auth/           client API authentication
//     |_ config/         static configuration objects
//     |  |_ global/      global configuration
//     |  |  |_ / GET     retrieve all global configuration
//     |  |  |_ / PUT     modify global configuration
//     |  |_ vrouter/     virtual router(s) configuration
//     |  |  |_ / GET     retrieve all virtual router configuration
//     |  |  |_ / PUT     modify virtual router configuration
//     |  |_ protocols/   protocols configuration
//     |     |_ / GET     retrieve all protocols configuration
//     |     |_ / PUT     modify protocols configuration
//     |_ run/            running configurations objects
//        |_ global/      running global configuration
//        |  |_ / GET     retrieve running global configuration
//        |_ vrrp/
//        |  |_ / GET      retrieve all VRRP information
//        |  |_ / PUT      modify a specific virtual router (spec. grp/intf)
//        |  |_ / POST     add a new VRRP virtual router (spec. grp/intf)
//        |  |_ / DELETE   remove a specific virtual router (spec. grp/intf)
//        |  |_ <group-id>/
//        |     |_ / GET       retrieve group specific information
//        |     |_ / PUT       modify specific virtual router (spec. intf)
//        |     |_ / POST      add a new VRRP virtual router (spec. intf)
//        |     |_ / DELETE    remove a specific virtual router (spec. intf)
//        |     |_ /<interface>/
//        |         |_ / GET       get specific virtual router information
//        |         |_ / PUT       modify specific virtual router
//        |         |_ / POST      add a new VRRP virtual router
//        |         |_ / DELETE    remove a specific virtual router
//        |_ protocols/
//           |_ / GET          retrieve all protocols information
//           |_ static/
//              |_ / GET       retrieve all static routes
//              |_ / POST      add a new static route (specify route)
//              |_ / PUT       modify a static route (specify route)
//              |_ / DELETE    remove a static route (specifc route)
//

// router() function
fn router(down_api: &DownstreamAPI) -> Router {
    // create new pipeline
    let pipeline = new_pipeline();

    // add CookieParser middleware
    let pipeline = pipeline.add(CookieParser);

    // create the state middleware to share the downstream API
    let stm = StateMiddleware::new(down_api.clone()); // verify

    // add state middleware to existing pipeline
    let pipeline = pipeline.add(stm);

    // construct a basic chain from the pipeline
    let (chain, pipelines) = single_pipeline(pipeline.build());

    // build the router with chain and pipelines
    build_router(chain, pipelines, |route| {
        // index
        route.get_or_head("/").to(handlers::index);

        // auth/ scope
        route.scope("/auth", |route| {
            // / (POST)
            route.post("/").to(handlers::auth::client);
        });

        // config/ scope
        route.scope("/config", |route| {
            // global/
            route.scope("/global", |route| {
                route.get("/").to(handlers::config::global::all);
            });
            // vrouter/
            route.scope("/vrouter", |route| {
                route.get("/").to(handlers::config::vrouter::all);
            });
            // protocols/
            route.scope("/protocols", |route| {
                route.get("/").to(handlers::config::protocols::all);
            })
        });

        // run/ scope
        route.scope("/run", |route| {
            // global/ scope
            route.scope("/global", |route| {
                route.get("/").to(handlers::run::global::all);
            });
            // vrrp/ scope
            route.scope("/vrrp", |route| {
                // /
                route.get("/").to(handlers::run::vrrp::all);
                // <group-id>/
                route
                    .get("/:group_id")
                    .with_path_extractor::<GroupIdExtractor>()
                    .to(handlers::run::vrrp::group);
                // <group-id>/<interface>/
                route
                    .get("/:group_id/:interface")
                    .with_path_extractor::<GroupIdInterfaceExtractor>()
                    .to(handlers::run::vrrp::group_interface);
            });
            // protocols/ scope
            route.scope("/protocols", |route| {
                // /
                route.get("/").to(handlers::run::protocols::all);
                // static/
                route.get("/static").to(handlers::run::protocols::pstatic);
            });
        });
    })
}

// start() function
pub fn start(down_api: DownstreamAPI, host: String, tls: bool, tls_key: String, tls_cert: String) {
    println!("Client API Server listening on http://{}", host);

    // if TLS is enabled
    if tls {
        let acceptor = build_tls_acceptor(tls_key, tls_cert).unwrap();
        let sockaddr = host
            .to_socket_addrs()
            .unwrap()
            .next()
            .ok_or_else(|| err_msg("Invalid Socket Address"))
            .unwrap();
        let listener = TcpListener::bind(&sockaddr).unwrap();
        let server = bind_server(
            listener,
            move || Ok(router(&down_api)),
            move |socket| {
                acceptor
                    .accept_async(socket)
                    .map_err(|e| println!("OpenSSL error: {}", e))
            },
        );
        let mut runtime = Runtime::new().unwrap();
        runtime
            .block_on(server)
            .map_err(|()| err_msg("Server failed"))
            .unwrap();
    } else {
        gotham::start(host, router(&down_api))
    }
}

// build_tls_acceptor() function
fn build_tls_acceptor(keyfile: String, certfile: String) -> Result<SslAcceptor, Error> {
    // openssl req -new -x509 -sha256 -newkey rsa:2048 -nodes -keyout key.pem -days 365 -out cert.pem
    let key = std::fs::read(keyfile).expect("Cannot read RSA key file");
    let cert = std::fs::read(certfile).expect("Cannot read X.509 certificate file");
    let cert = X509::from_pem(&cert).expect("Malformed X.509 certificate");
    let key = PKey::private_key_from_pem(&key).expect("Malformed PEM key");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_certificate(&cert)?;
    builder.set_private_key(&key)?;

    let acceptor = builder.build();
    Ok(acceptor)
}

// serialize_answer() function
fn serialize_answer<T: Serialize>(state: &State, ans: T) -> Response<Body> {
    create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_JSON,
        serde_json::to_vec(&ans).expect("serialized response"),
    )
}

// GroupIdExtractor structure
#[derive(Deserialize, StateData, StaticResponseExtender)]
struct GroupIdExtractor {
    group_id: u8,
}

// GroupIdInterfaceExtractor structure
#[derive(Deserialize, StateData, StaticResponseExtender)]
struct GroupIdInterfaceExtractor {
    group_id: u8,
    interface: String,
}

// read_session_cookies() function
pub fn read_session_cookies(state: &State) -> (String, u64, u64, String) {
    // retrieve session cookies
    let c = CookieJar::borrow_from(&state);
    let user = {
        c.get(COOKIE_USER)
            .map(|c| c.value().to_owned())
            .unwrap_or_else(|| "null".to_string())
    };
    let ts_since: u64 = {
        c.get(COOKIE_TIMESTAMP)
            .map(|c| c.value().parse::<u64>().unwrap())
            .unwrap_or_else(|| 0)
    };
    let nonce: u64 = {
        c.get(COOKIE_NONCE)
            .map(|c| c.value().parse::<u64>().unwrap())
            .unwrap_or_else(|| 0)
    };
    let token = {
        c.get(COOKIE_TOKEN)
            .map(|c| c.value().to_owned())
            .unwrap_or_else(|| "null".to_string())
    };
    (user, ts_since, nonce, token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn receive_hello_response() {
        let down_api = DownstreamAPI::new();

        let server = TestServer::new(router(&down_api)).unwrap();
        let response = server.client().get("http://localhost").perform().unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
