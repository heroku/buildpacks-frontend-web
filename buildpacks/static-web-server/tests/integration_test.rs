// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]
#![allow(clippy::unwrap_used)]

use libcnb_test::assert_contains;
use test_support::{
    assert_web_response, retry, start_container, static_web_server_integration_test,
    DEFAULT_RETRIES, DEFAULT_RETRY_DELAY,
};

#[test]
#[ignore = "integration test"]
fn no_project_toml() {
    static_web_server_integration_test("./fixtures/no_project_toml", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(&ctx, |_container, socket_addr| {
            // Test for successful response
            let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/")).call()
            })
            .unwrap();
            let response_status = response.status();
            assert_eq!(response_status, 200);
            let response_body = response.into_string().unwrap();
            assert_contains!(response_body, "Welcome to CNB Static Web Server Test!");

            // Test for default Not Found response
            let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/non-existent-path")).call()
            });
            match response_result {
                Err(ureq::Error::Status(code, response)) => {
                    assert_eq!(code, 404);
                    let h = response.header("Content-Type").unwrap_or_default();
                    assert_contains!(h, "text/html");
                    let response_body = response.into_string().unwrap();
                    assert_contains!(response_body, "404 Not Found");
                }
                Ok(_) => {
                    panic!("should respond 404 Not Found, but got 200 ok");
                }
                Err(error) => {
                    panic!("should respond 404 Not Found, but got other error: {error:?}");
                }
            }
        });
    });
}

#[test]
#[ignore = "integration test"]
fn custom_doc_root() {
    static_web_server_integration_test("./fixtures/custom_doc_root", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Static Web Server Doc Root Test!");
    });
}

#[test]
#[ignore = "integration test"]
fn custom_index() {
    static_web_server_integration_test("./fixtures/custom_index", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Static Web Server Custom Index Test!");
    });
}

#[test]
#[ignore = "integration test"]
fn top_level_doc_root() {
    static_web_server_integration_test("./fixtures/top_level_doc_root", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(&ctx, |_container, socket_addr| {
            let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/")).call()
            });
            match response_result {
                Ok(response) => {
                    assert_eq!(response.status(), 200);
                    let h = response.header("Content-Type").unwrap_or_default();
                    assert_contains!(h, "text/html");
                    let response_body = response.into_string().unwrap();
                    assert_contains!(
                        response_body,
                        "Welcome to CNB Static Web Server Top-level Doc Root Test!"
                    );
                }
                Err(error) => {
                    panic!("should respond 200 ok, but got other error: {error:?}");
                }
            }

            let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/non-existent-path")).call()
            });
            match response_result {
                Err(ureq::Error::Status(code, response)) => {
                    assert_eq!(code, 404);
                    let h = response.header("Content-Type").unwrap_or_default();
                    assert_contains!(h, "text/html");
                    let response_body = response.into_string().unwrap();
                    assert_contains!(response_body, "Custom 404");
                }
                Ok(_) => {
                    panic!("should respond 404 Not Found, but got 200 ok");
                }
                Err(error) => {
                    panic!("should respond 404 Not Found, but got other error: {error:?}");
                }
            }
        });
    });
}

#[test]
#[ignore = "integration test"]
fn custom_headers() {
    static_web_server_integration_test("./fixtures/custom_headers", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(&ctx, |_container, socket_addr| {
            let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/")).call()
            })
            .unwrap();
            let h = response.header("X-Global").unwrap_or_default();
            assert_contains!(h, "Hello");
            let h = response.header("X-Only-Default").unwrap_or_default();
            assert_contains!(h, "Hiii");
            assert!(
                !response
                    .headers_names()
                    .contains(&String::from("X-Only-HTML")),
                "should not include X-Only-HTML header"
            );

            let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/page2.html")).call()
            })
            .unwrap();
            let h = response.header("X-Only-HTML").unwrap_or_default();
            assert_contains!(h, "Hi");
            assert!(
                !response
                    .headers_names()
                    .contains(&String::from("X-Only-Default")),
                "should not include X-Only-Default header"
            );
        });
    });
}

#[test]
#[ignore = "integration test"]
fn custom_errors() {
    static_web_server_integration_test("./fixtures/custom_errors", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(&ctx, |_container, socket_addr| {
            let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/non-existent-path")).call()
            });
            match response_result {
                Err(ureq::Error::Status(code, response)) => {
                    assert_eq!(code, 404);
                    let h = response.header("Content-Type").unwrap_or_default();
                    assert_contains!(h, "text/html");
                    let response_body = response.into_string().unwrap();
                    assert_contains!(response_body, "Custom 404");
                }
                Ok(_) => {
                    panic!("should respond 404 Not Found, but got 200 ok");
                }
                Err(error) => {
                    panic!("should respond 404 Not Found, but got other error: {error:?}");
                }
            }
        });
    });
}

#[test]
#[ignore = "integration test"]
fn client_side_routing() {
    static_web_server_integration_test("./fixtures/client_side_routing", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(&ctx, |_container, socket_addr| {
            let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                ureq::get(&format!("http://{socket_addr}/non-existent-path")).call()
            });
            match response_result {
                Ok(response) => {
                    assert_eq!(response.status(), 200);
                    let h = response.header("Content-Type").unwrap_or_default();
                    assert_contains!(h, "text/html");
                    let response_body = response.into_string().unwrap();
                    assert_contains!(
                        response_body,
                        "Welcome to CNB Static Web Server Client Side Routing Test!"
                    );
                }
                Err(error) => {
                    panic!("should respond 200 ok, but got other error: {error:?}");
                }
            }
        });
    });
}
