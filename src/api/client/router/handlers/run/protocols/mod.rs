//! Client API - protocols running configuration handlers
use super::*;

/// all() handler function
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
    let q = ClientAPIQuery::RunProtoAll(sess);
    down.query(q);

    // read answer and set HTTP body (blocking)
    let htbody = {
        match down.read() {
            // if a response is returned
            ClientAPIResponse::RunProtoAll(ans) => serialize_answer(&state, ans),
            ClientAPIResponse::Unauthorized => {
                create_empty_response(&state, StatusCode::UNAUTHORIZED)
            }
            _ => create_response(
                &state,
                StatusCode::INTERNAL_SERVER_ERROR,
                mime::TEXT_PLAIN,
                NOT_FOUND,
            ),
        }
    };
    return (state, htbody);
}

// pstatic() handler function
pub fn pstatic(state: State) -> (State, Response<Body>) {
    // borrow references to the Downstream API
    let down = DownstreamAPI::borrow_from(&state);

    // send a query downstream
    let sess = SessionToken::new();
    // send a query downstream
    let q = ClientAPIQuery::RunProtoStatic(sess);
    down.query(q);

    // read answer and set HTTP body (blocking)
    let htbody = {
        match down.read() {
            // if a response is returned
            ClientAPIResponse::RunProtoStatic(ans) => serialize_answer(&state, ans),
            ClientAPIResponse::Unauthorized => {
                create_empty_response(&state, StatusCode::UNAUTHORIZED)
            }
            _ => create_response(
                &state,
                StatusCode::INTERNAL_SERVER_ERROR,
                mime::TEXT_PLAIN,
                NOT_FOUND,
            ),
        }
    };
    return (state, htbody);
}
