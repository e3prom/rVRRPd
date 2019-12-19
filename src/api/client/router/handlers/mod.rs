//! Client API router handlers module
use super::*;

// gotham
use gotham::state::State;

// authentication scope handlers
pub mod auth;

// configuration scope handlers
pub mod config;

// running config scope handlers
pub mod run;

// index() function
pub fn index(state: State) -> (State, Response<Body>) {
    let body = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                <title>rVRRPd Client API</title>
            </head>
            <body>
                <h2>Welcome to the rVRRPd Client API HTTP Interface</h2>
                <p>If you see this page, it means the API processes requests from you.<p>
                <p>However, you must first <a href=/auth>authenticate</a> by sending a POST request
                with a valid <code>user</code> and <code>password</code> key = value pair.</p>
                <p>Once authenticated, you will receive a cookie to be used for your further
                resources requests.</p>
                </p>
            </body>
        </html>
    "#;
    //  HTTP body
    let htbody = { create_response(&state, StatusCode::OK, mime::TEXT_HTML, body) };
    return (state, htbody);
}
