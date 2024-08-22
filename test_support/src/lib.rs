// This module is only used for testing, where using unwrap() is acceptable.
#![allow(clippy::unwrap_used)]

use libcnb::data::buildpack_id;
use libcnb_test::{
    assert_contains, BuildConfig, BuildpackReference, ContainerConfig, ContainerContext,
    TestContext, TestRunner,
};
use std::net::SocketAddr;
use std::panic;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const DEFAULT_BUILDER: &str = "heroku/builder:22";
pub const PORT: u16 = 8080;
pub const DEFAULT_RETRIES: u32 = 10;
pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);

#[must_use]
fn get_integration_test_builder() -> String {
    std::env::var("INTEGRATION_TEST_CNB_BUILDER").unwrap_or(DEFAULT_BUILDER.to_string())
}

pub fn static_web_server_integration_test(fixture: &str, test_body: fn(TestContext)) {
    static_web_server_integration_test_with_config(fixture, |_| {}, test_body);
}

pub fn static_web_server_integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
) {
    integration_test_with_config(
        fixture,
        with_config,
        test_body,
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/static-web-server"
        ))],
    );
}

pub fn website_integration_test(fixture: &str, test_body: fn(TestContext)) {
    website_integration_test_with_config(fixture, |_| {}, test_body);
}

pub fn website_integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
) {
    integration_test_with_config(
        fixture,
        with_config,
        test_body,
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/website"
        ))],
    );
}

fn integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
    buildpacks: &[BuildpackReference],
) {
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("The CARGO_MANIFEST_DIR should be automatically set by Cargo when running tests but it was not");

    let builder = get_integration_test_builder();
    let app_dir = cargo_manifest_dir.join("tests").join(fixture);

    // TODO: Once Pack build supports `--platform` and libcnb-test adjusted accordingly, change this
    // to allow configuring the target arch independently of the builder name (eg via env var).
    let target_triple = match builder.as_str() {
        // Compile the buildpack for ARM64 iff the builder supports multi-arch and the host is ARM64.
        "heroku/builder:24" if cfg!(target_arch = "aarch64") => "aarch64-unknown-linux-musl",
        _ => "x86_64-unknown-linux-musl",
    };

    let mut build_config = BuildConfig::new(builder, app_dir);
    build_config.buildpacks(buildpacks);
    build_config.target_triple(target_triple);
    with_config(&mut build_config);

    TestRunner::default().build(build_config, test_body);
}

pub fn retry<T, E>(
    attempts: u32,
    retry_delay: Duration,
    retryable_action: impl Fn() -> Result<T, E>,
) -> Result<T, E> {
    let mut result = retryable_action();
    for _ in 1..attempts {
        if result.is_ok() {
            return result;
        }
        std::thread::sleep(retry_delay);
        result = retryable_action();
    }
    result
}

pub fn start_container(ctx: &TestContext, in_container: impl Fn(&ContainerContext, &SocketAddr)) {
    ctx.start_container(
        ContainerConfig::new()
            .env("PORT", PORT.to_string())
            .expose_port(PORT),
        |container| {
            let socket_addr = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                std::panic::catch_unwind(|| container.address_for_port(PORT))
            })
            .unwrap();
            in_container(&container, &socket_addr);
            let container_logs = container.logs_now();
            println!(
                "
------ begin container logs (stderr) ------
{}------ end (stderr) & begin (stdout) ------
{}------ end container logs ------",
                container_logs.stderr, container_logs.stdout
            );
        },
    );
}

pub fn assert_web_response(ctx: &TestContext, expected_response_body: &'static str) {
    start_container(ctx, |_container, socket_addr| {
        let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
            ureq::get(&format!("http://{socket_addr}/")).call()
        })
        .unwrap();
        let response_body = response.into_string().unwrap();
        assert_contains!(response_body, expected_response_body);
    });
}

pub fn wait_for<F>(condition: F, max_wait_time: Duration)
where
    F: Fn() + panic::RefUnwindSafe,
{
    let mut error = None;
    let start_time = SystemTime::now();
    while SystemTime::now()
        .duration_since(start_time)
        .expect("should not be an earlier time")
        < max_wait_time
    {
        match panic::catch_unwind(&condition) {
            Ok(()) => return,
            Err(err) => error = Some(err),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    match error {
        None => panic!("timeout exceeded"),
        Some(error) => panic::resume_unwind(error),
    }
}
