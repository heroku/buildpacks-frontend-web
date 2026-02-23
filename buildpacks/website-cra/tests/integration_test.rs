// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{assert_contains, ContainerConfig};
use test_support::{
    retry, start_container, website_cra_integration_test, DEFAULT_RETRIES, DEFAULT_RETRY_DELAY,
};

#[test]
#[ignore = "integration test"]
fn cra_app() {
    website_cra_integration_test("./fixtures/cra_app", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (create-react-app) Buildpack");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/")).call()
                })
                .unwrap();
                let response_body = response.into_string().unwrap();
                assert_contains!(response_body, "Web site created using create-react-app");

                let second_response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/client-side-route")).call()
                })
                .unwrap();
                let second_response_body = second_response.into_string().unwrap();
                assert_contains!(
                    second_response_body,
                    "Web site created using create-react-app"
                );
            },
        );
    });
}
