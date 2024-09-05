// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::assert_contains;
use test_support::{assert_web_response, website_nodejs_integration_test};

#[test]
#[ignore = "integration test"]
fn ember_cli_app() {
    website_nodejs_integration_test("./fixtures/ember_cli_app", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (Ember.js)");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "cnb-ember-web-app/config/environment");
    });
}
