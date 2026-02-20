// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{assert_contains, ContainerConfig};
use test_support::{
    retry, start_container, website_ember_integration_test, DEFAULT_RETRIES, DEFAULT_RETRY_DELAY,
};

#[test]
#[ignore = "integration test"]
fn ember_cli_app() {
    website_ember_integration_test("./fixtures/ember_cli_app", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (Ember.js)");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            ContainerConfig::new().env(
                "PUBLIC_WEB_INTEGRATION_TEST",
                "runtime-config-via-container-env",
            ),
            |_container, socket_addr| {
                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/")).call()
                })
                .unwrap();
                let response_body = response.into_string().unwrap();

                // This is a unique part of the Ember app's rendered HTML.
                assert_contains!(response_body, "ember-cli-app/config/environment");
                assert_contains!(
                    response_body,
                    r#"data-public_web_integration_test="runtime-config-via-container-env""#
                );

                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/test-client-side-routing")).call()
                })
                .unwrap();
                let response_body = response.into_string().unwrap();

                // This is a unique part of the Ember app's rendered HTML.
                assert_contains!(response_body, "ember-cli-app/config/environment");
                assert_contains!(
                    response_body,
                    r#"data-public_web_integration_test="runtime-config-via-container-env""#
                );
            },
        );
    });
}
