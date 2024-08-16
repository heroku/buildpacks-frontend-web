// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{assert_contains};
use test_support::{
    assert_web_response, website_integration_test,
    wait_for, PORT,
};

#[test]
#[ignore]
fn public_html() {
    website_integration_test("./fixtures/public_html", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (Public HTML)");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Website Test!");
    });
}
