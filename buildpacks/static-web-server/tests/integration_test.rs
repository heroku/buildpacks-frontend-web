// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{assert_contains};
use test_support::{
    assert_web_response, assert_web_response_header, static_web_server_integration_test,
    wait_for, PORT,
};

#[test]
#[ignore = "integration test"]
fn no_project_toml() {
    static_web_server_integration_test("./fixtures/no_project_toml", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Static Web Server Test!");
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
fn custom_headers() {
    static_web_server_integration_test("./fixtures/custom_headers", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Static Web Server Response Headers Test!");
        assert_web_response_header(&ctx, "/", "X-Global", "Hello");
        assert_web_response_header(&ctx, "/", "X-Only-Default", "Hiii");
        assert_web_response_header(&ctx, "/page2.html", "X-Only-HTML", "Hi");
    });
}
