//! Client API - authentication handlers
use super::*;

// futures
use futures::{future, Future, Stream};

// gotham
use gotham::handler::{HandlerFuture, IntoHandlerError};

// regex
extern crate regex;
use regex::Regex;

/// client() handler function
pub fn client(mut state: State) -> Box<HandlerFuture> {
    // extract authentication information from HTTP POST
    let f = Body::take_from(&mut state)
        .concat2()
        .then(|body| match body {
            Ok(valid_body) => {
                // extract body content
                let content = String::from_utf8(valid_body.to_vec()).unwrap();
                // authenticate the user to the Client API
                let down = DownstreamAPI::borrow_from(&state);
                let q = match regex_captures_authav(&content) {
                    // if the regexp captured the user/passwd attributes
                    Some(c) => ClientAPIQuery::AuthRequest(
                        c.get(1).unwrap().as_str().to_string(),
                        c.get(2).unwrap().as_str().to_string(),
                    ),
                    // if not, return an error
                    None => {
                        let resp = create_empty_response(&state, StatusCode::BAD_REQUEST);
                        return future::ok((state, resp));
                    }
                };
                // send authentication request
                down.query(q);

                // read downstream API channel
                match down.read() {
                    // read authentication response
                    ClientAPIResponse::AuthResponse(sess) => {
                        match sess {
                            // authentication succeeded
                            Some(st) => {
                                // build cookies
                                let cookie_user = Cookie::build(COOKIE_USER, st.user())
                                    .http_only(true)
                                    .finish();
                                let cookie_ts = Cookie::build(COOKIE_TIMESTAMP, st.ts_since())
                                    .http_only(true)
                                    .finish();
                                let cookie_nonce = Cookie::build(COOKIE_NONCE, st.nonce())
                                    .http_only(true)
                                    .finish();
                                let cookie_token = Cookie::build(COOKIE_TOKEN, st.token())
                                    .http_only(true)
                                    .finish();
                                // create an empty response
                                let mut resp = create_empty_response(&state, StatusCode::OK);
                                // append cookies to response
                                resp.headers_mut()
                                    .append(SET_COOKIE, cookie_user.to_string().parse().unwrap());
                                resp.headers_mut()
                                    .append(SET_COOKIE, cookie_ts.to_string().parse().unwrap());
                                resp.headers_mut()
                                    .append(SET_COOKIE, cookie_nonce.to_string().parse().unwrap());
                                resp.headers_mut()
                                    .append(SET_COOKIE, cookie_token.to_string().parse().unwrap());
                                // return future
                                future::ok((state, resp))
                            }
                            // no token has been issued
                            None => {
                                let resp = create_empty_response(
                                    &state,
                                    StatusCode::UNAUTHORIZED,
                                );
                                return future::ok((state, resp));
                            }
                        }
                    }
                    // other response types are considered invalid
                    _ => {
                        let resp = create_empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
                        return future::ok((state, resp));
                    }
                }
            }
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(f)
}

/// regex_captures_authav() function
/// creates a globally accessible and static compiled regular expression
fn regex_captures_authav(content: &String) -> Option<regex::Captures> {
    // only allow alphanumeric user strings with at least one character, up to 64
    // allow passwords with less than 256 printable characters (is not reflected in Cookies)
    lazy_static! {
        static ref REGEX_HTBODY_AUTH_AV: Regex =
            Regex::new(r"^user=(?P<u>[[:alnum:]]{1,64}) passwd=(?P<p>[[:print:]]{1,256})$")
                .unwrap();
    }
    // capture content using the pre-compiled regular expression
    REGEX_HTBODY_AUTH_AV.captures(content)
}
