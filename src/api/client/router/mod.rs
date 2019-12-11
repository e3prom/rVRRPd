//! Client API router
use super::*;

// gotham
extern crate gotham;
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};

// hyper
extern crate hyper;
use hyper::{Body, Response, StatusCode};

// mime
extern crate mime;

// serde
use serde::Serialize;

// handlers
mod handlers;

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
fn router(down_api: DownstreamAPI) -> Router {
    // create the state middleware to share the downstream API
    let middleware = StateMiddleware::new(down_api);

    // create a middleware pipeline
    let pipeline = single_middleware(middleware);

    // construct a basic chain from the pipeline
    let (chain, pipelines) = single_pipeline(pipeline);

    // build the router with chain and pipelines
    build_router(chain, pipelines, |route| {
        // index
        route.get_or_head("/").to(handlers::index);

        // // auth/ scope
        // route.scope("/auth", |route| {
        //     route.get("/client").to(auth::client);
        // });

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
pub fn start(down_api: DownstreamAPI) {
    let addr = "0.0.0.0:7080";
    println!("Client API Server listening on http://{}", addr);
    gotham::start(addr, router(down_api))
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

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn receive_hello_response() {
        let down_api = DownstreamAPI::new();

        let server = TestServer::new(router(down_api)).unwrap();
        let response = server.client().get("http://localhost").perform().unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_body().unwrap();
        assert_eq!(&body[..], b"Hello and welcome");
    }
}
