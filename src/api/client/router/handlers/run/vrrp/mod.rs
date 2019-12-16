//! Client API - VRRP running configuration handlers
use super::*;

// all() handler function
pub fn all(state: State) -> (State, Response<Body>) {
    // borrow references to the Downstream API
    let down = DownstreamAPI::borrow_from(&state);

    // retrieve session cookie
    let (user, ts_since, nonce, token) = read_session_cookies(&state);

    // create SessionToken
    let mut sess = SessionToken::new();
    sess.set_user(user);
    sess.set_tssince(ts_since);
    sess.set_nonce(nonce);
    sess.set_token(token);

    // send a query downstream
    let q = ClientAPIQuery::RunVRRPAll(sess);
    down.query(q);

    // read answer and set HTTP body (blocking)
    let htbody = {
        match down.read() {
            // if a response is returned
            ClientAPIResponse::RunVRRPAll(ans) => serialize_answer(&state, ans),
            _ => create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, NOT_FOUND),
        }
    };
    return (state, htbody);
}

// group() handler function
pub fn group(state: State) -> (State, Response<Body>) {
    // borrow references to the Downstream API
    let down = DownstreamAPI::borrow_from(&state);

    // extract group_id from GET path
    let path = GroupIdExtractor::borrow_from(&state);
    let gid = path.group_id;

    // send a query downstream
    let sess = SessionToken::new();
    let q = ClientAPIQuery::RunVRRPGrp(sess, gid);
    down.query(q);

    // read answer and set HTTP body (blocking)
    let htbody = {
        match down.read() {
            // if a response is returned
            ClientAPIResponse::RunVRRPGrp(ans) => serialize_answer(&state, ans),
            _ => create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, NOT_FOUND),
        }
    };
    return (state, htbody);
}

/// group_interface() handler function
pub fn group_interface(state: State) -> (State, Response<Body>) {
    // borrow references to the Downstream API
    let down = DownstreamAPI::borrow_from(&state);

    // retrieve session cookie
    let (user, ts_since, nonce, token) = read_session_cookies(&state);

    // create SessionToken
    let mut sess = SessionToken::new();
    sess.set_user(user);
    sess.set_tssince(ts_since);
    sess.set_nonce(nonce);
    sess.set_token(token);

    // extract group_id and interface from GET path
    let path = GroupIdInterfaceExtractor::borrow_from(&state);
    let gid = path.group_id;
    let intf = path.interface.clone();

    // send a query downstream
    let sess = SessionToken::new();
    let q = ClientAPIQuery::RunVRRPGrpIntf(sess, gid, intf);
    down.query(q);

    // read answer and set HTTP body (blocking)
    let htbody = {
        match down.read() {
            // if a response is returned
            ClientAPIResponse::RunVRRPGrpIntf(ans) => serialize_answer(&state, ans),
            _ => create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, NOT_FOUND),
        }
    };
    return (state, htbody);
}
