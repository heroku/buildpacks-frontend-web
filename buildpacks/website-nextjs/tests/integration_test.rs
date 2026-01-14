// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{assert_contains, ContainerConfig};
use test_support::{
    retry, start_container, website_nodejs_integration_test, DEFAULT_RETRIES, DEFAULT_RETRY_DELAY,
};

#[test]
#[ignore = "integration test"]
fn nextjs_app() {
    website_nodejs_integration_test("./fixtures/next_app", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (Next.js)");
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

                assert_contains!(
                    response_body,
                    r#"data-public_web_integration_test="runtime-config-via-container-env""#
                );
            },
        );
    });
}
