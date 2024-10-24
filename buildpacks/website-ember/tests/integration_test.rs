// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use std::{fs, os::unix::fs::PermissionsExt};

use libcnb_test::{assert_contains, ContainerConfig};
use tempfile::tempdir;
use test_support::{
    retry, start_container, start_container_entrypoint, website_nodejs_integration_test,
    DEFAULT_RETRIES, DEFAULT_RETRY_DELAY,
};
use uuid::Uuid;

#[test]
#[ignore = "integration test"]
fn ember_cli_app() {
    website_nodejs_integration_test("./fixtures/ember_cli_app", |ctx| {
        let unique = Uuid::new_v4();

        let temp_dir = tempdir().expect("should create temporary directory for artifact storage");
        let temp_sub_dir = "static-artifacts-storage";
        let local_storage_path = temp_dir.path().join(temp_sub_dir);
        println!("local_storage_path: {local_storage_path:?}");

        // Workaround for GitHub Runner & Docker container not running with same gid/uid/permissions:
        // create & set the temp local storage dir permissions to be world-accessible.
        fs::create_dir_all(&local_storage_path)
            .expect("local_storage_path directory should be created");
        let mut perms = fs::metadata(&local_storage_path)
            .expect("local dir already exists")
            .permissions();
        perms.set_mode(0o777);
        fs::set_permissions(&local_storage_path, perms).expect("local dir permission can be set");

        let container_volume_path = "/static-artifacts-storage";
        let container_volume_url = "file://".to_owned() + container_volume_path;

        assert_contains!(ctx.pack_stdout, "Website (Ember.js)");
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        assert_contains!(ctx.pack_stdout, "Release Phase");
        assert_contains!(
            ctx.pack_stdout,
            "Not running `build` as it was disabled by a participating buildpack"
        );

        start_container_entrypoint(
            &ctx,
            ContainerConfig::new()
                .env("RELEASE_ID", unique)
                .env("STATIC_ARTIFACTS_URL", &container_volume_url)
                .bind_mount(&local_storage_path, container_volume_path),
            &"release".to_string(),
            |container| {
                let log_output = container.logs_now();
                assert_contains!(log_output.stderr, "release-phase plan");
                assert_contains!(
                    log_output.stdout,
                    format!("save-release-artifacts writing archive: release-{unique}.tgz")
                        .as_str()
                );
                assert_contains!(log_output.stderr, "release-phase complete.");
            },
        );
        start_container(
            &ctx,
            ContainerConfig::new()
                .env("RELEASE_ID", unique)
                .env("STATIC_ARTIFACTS_URL", &container_volume_url)
                .bind_mount(&local_storage_path, container_volume_path),
            |container, socket_addr| {
                let log_output = container.logs_now();
                assert_contains!(
                    log_output.stderr,
                    format!("load-release-artifacts reading archive: release-{unique}.tgz")
                        .as_str(),
                );
                assert_contains!(log_output.stderr, "load-release-artifacts complete.");

                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/")).call()
                })
                .unwrap();
                let response_body = response.into_string().unwrap();

                // This is a unique part of the Ember app's rendered HTML.
                assert_contains!(response_body, "cnb-ember-web-app/config/environment");

                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/test-client-side-routing")).call()
                })
                .unwrap();
                let response_body = response.into_string().unwrap();

                // This is a unique part of the Ember app's rendered HTML.
                assert_contains!(response_body, "cnb-ember-web-app/config/environment");
            },
        );
    });
}
