//! Client API - virtual routers configuration handlers
use super::*;

// all() handler function
pub fn all(state: State) -> (State, Response<Body>) {
    // borrow references to the Downstream API
    let down = DownstreamAPI::borrow_from(&state);

    // send a query downstream
    let q = ClientAPIQuery::CfgVrrpAll;
    down.query(q);

    // read answer (blocking)
    let htbody = {
        match down.read() {
            // if a response is returned
            ClientAPIResponse::CfgVrrpAll(ans) => serialize_answer(&state, ans),
            _ => create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, NOT_FOUND),
        }
    };
    return (state, htbody);
}
