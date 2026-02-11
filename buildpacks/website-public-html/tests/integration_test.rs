// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::assert_contains;
use test_support::{assert_web_response, website_public_html_integration_test};

#[test]
#[ignore = "integration test"]
fn public_html_sequence() {
    // Run these in sequence to avoid this weird pack filesystem registry collision:
    // lookup buildpack 'urn:cnb:registry:heroku/release-phase': refreshing cache: initializing (/home/runner/.pack/registry-a932275bd19c2d9e1b88fa06698fd2f5427a363d25bf87fa500691c373089381): rebuilding registry cache: rename /home/runner/.pack/registry2121277531 /home/runner/.pack/registry-a932275bd19c2d9e1b88fa06698fd2f5427a363d25bf87fa500691c373089381: file exists
    public_html();
    custom_root();
}

fn public_html() {
    website_public_html_integration_test("./fixtures/public_html", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (Public HTML)");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Website Test!");
    });
}

fn custom_root() {
    website_public_html_integration_test("./fixtures/custom_root", |ctx| {
        assert_contains!(ctx.pack_stdout, "Website (Public HTML)");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_web_response(&ctx, "Welcome to CNB Website with Configured Root Test!");
    });
}
