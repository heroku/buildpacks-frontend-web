// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]
#![allow(clippy::unwrap_used)]

use libcnb_test::{assert_contains, assert_contains_match, ContainerConfig};
use test_support::{
    assert_web_response, retry, start_container, static_web_server_integration_test,
    DEFAULT_RETRIES, DEFAULT_RETRY_DELAY,
};

#[test]
#[ignore = "integration test"]
fn default_behavior() {
    static_web_server_integration_test("./fixtures/no_project_toml", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}")).call()
                });
                match response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let h = response.header("Content-Type").unwrap_or_default();
                        assert_contains!(h, "text/html");
                        let response_body = response.into_string().unwrap();
                        assert_contains!(response_body, "Welcome to CNB Static Web Server Test!");
                    }
                    Err(error) => {
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn build_command() {
    static_web_server_integration_test("./fixtures/build_command", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
                // Test for successful response
                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/")).call()
                })
                .unwrap();
                let response_status = response.status();
                assert_eq!(response_status, 200);
                let response_body = response.into_string().unwrap();
                assert_contains!(
                    response_body,
                    "Welcome to CNB Static Web Server Build Command Test!"
                );

                // Test for default Not Found response
                let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/test-output.txt")).call()
                })
                .unwrap();
                let response_status = response.status();
                assert_eq!(response_status, 200);
                let response_body = response.into_string().unwrap();
                assert_contains!(response_body, "Build Command Output Test!");
            },
        );
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
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
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
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn custom_headers() {
    static_web_server_integration_test("./fixtures/custom_headers", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
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
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn custom_errors() {
    static_web_server_integration_test("./fixtures/custom_errors", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
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
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn client_side_routing() {
    static_web_server_integration_test("./fixtures/client_side_routing", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |_container, socket_addr| {
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
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn runtime_configuration_custom() {
    static_web_server_integration_test("./fixtures/runtime_config", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            ContainerConfig::new().env(
                "PUBLIC_WEB_INTEGRATION_TEST",
                "runtime-config-via-container-env",
            ),
            |container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/")).call()
                });
                match response_result {
                    Ok(response) => {
                        let response_body = response.into_string().unwrap();
                        assert_contains!(
                            response_body,
                            r#"data-public_web_integration_test="runtime-config-via-container-env""#
                        );
                    }
                    Err(error) => {
                        panic!("should respond 200 Ok, but received: {error:?}");
                    }
                }
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/subsection/")).call()
                });
                match response_result {
                    Ok(response) => {
                        let response_body = response.into_string().unwrap();
                        assert_contains!(
                            response_body,
                            r#"data-public_web_integration_test="runtime-config-via-container-env""#
                        );
                    }
                    Err(error) => {
                        panic!("should respond 200 Ok, but received: {error:?}");
                    }
                }

                let log_output = container.logs_now();
                assert_contains!(
                    log_output.stderr,
                    "Runtime configuration written into 'public/index.html'"
                );
                assert_contains!(
                    log_output.stderr,
                    "Runtime configuration written into 'public/subsection/index.html'"
                );
                assert_contains!(
                    log_output.stderr,
                    "Runtime configuration skipping 'public/non-existent.html'"
                );
                assert_contains!(
                    log_output.stderr,
                    "Runtime configuration written into 'public/subsection/subsubsection/index.html'"
                );
                assert_contains!(
                    log_output.stderr,
                    "Runtime configuration written into 'public/subsection/subsubsection/second.html'"
                );
                assert_contains!(
                    log_output.stderr,
                    "Runtime configuration written into 'public/subsection/subsubsection/subsubsubsection/index.html'"
                );
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn runtime_configuration_default() {
    static_web_server_integration_test("./fixtures/no_project_toml", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            ContainerConfig::new().env(
                "PUBLIC_WEB_INTEGRATION_TEST",
                "runtime-config-via-container-env",
            ),
            |_container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/")).call()
                });
                match response_result {
                    Ok(response) => {
                        let response_body = response.into_string().unwrap();
                        assert_contains!(
                            response_body,
                            r#"data-public_web_integration_test="runtime-config-via-container-env""#
                        );
                    }
                    Err(error) => {
                        panic!("should respond 200 Ok, but received: {error:?}");
                    }
                }
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn caddy_csp_nonce() {
    static_web_server_integration_test("./fixtures/caddy_csp_nonce", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}")).call()
                });
                match response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let h = response
                            .header("Content-Security-Policy")
                            .unwrap_or_default();
                        assert_contains_match!(h, "nonce-[0-9a-f-]+");
                        let response_body = response.into_string().unwrap();
                        assert_contains_match!(response_body, r#"nonce="[0-9a-f-]+""#);
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn caddy_clean_urls() {
    static_web_server_integration_test("./fixtures/caddy_clean_urls", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |container, socket_addr| {
                let index_response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}")).call()
                });
                match index_response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let response_body = response.into_string().unwrap();
                        assert_contains!(response_body, "Clean URLs Test");
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
                let other_response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/other")).call()
                });
                match other_response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let response_body = response.into_string().unwrap();
                        assert_contains!(response_body, "Clean URLs (Other) Test");
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
                let nested_response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}/nested")).call()
                });
                match nested_response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let response_body = response.into_string().unwrap();
                        assert_contains!(response_body, "Clean URLs (Nested) Test");
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
                let nested_second_response_result =
                    retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                        ureq::get(&format!("http://{socket_addr}/nested/second")).call()
                    });
                match nested_second_response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let response_body = response.into_string().unwrap();
                        assert_contains!(response_body, "Clean URLs (Nested Second) Test");
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
                let nested_deeper_response_result =
                    retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                        ureq::get(&format!("http://{socket_addr}/nested/deeper")).call()
                    });
                match nested_deeper_response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let response_body = response.into_string().unwrap();
                        assert_contains!(response_body, "Clean URLs (Nested Deeper) Test");
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn caddy_access_logs() {
    static_web_server_integration_test("./fixtures/caddy_access_logs", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            &mut ContainerConfig::new(),
            |container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}")).call()
                });
                match response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let logs = container.logs_now().to_string();
                        assert_contains!(logs, "\"logger\":\"http.log.access.public\"");
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn caddy_basic_auth() {
    static_web_server_integration_test("./fixtures/caddy_basic_auth", |ctx| {
        assert_contains!(ctx.pack_stdout, "Static Web Server");
        start_container(
            &ctx,
            ContainerConfig::new()
                .env("WEB_BASIC_AUTH_USERNAME", "visitor")
                .env(
                    "WEB_BASIC_AUTH_PASSWORD_BCRYPT",
                    // bcrypt password hash generated with
                    //   htpasswd -bnBC 10 "" openseasame | tr -d ':\n'
                    "$2y$10$Uc4licEvYDo2DqtNnTkfV.o0Bvr4Sqw0Vpdh7UMrmkqwTZ92EMkZ6",
                ),
            |container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}")).call()
                });
                match response_result {
                    Err(ureq::Error::Status(code, _response)) => {
                        assert_eq!(code, 401);
                    }
                    Ok(_) => {
                        panic!("should respond 401 Unauthorized, but got 200 ok");
                    }
                    Err(error) => {
                        panic!("should respond 401 Unauthorized, but got other error: {error:?}");
                    }
                }

                let auth_response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}"))
                        .set(
                            "Authorization",
                            // encoded basic auth data generated with that password:
                            //   echo -n 'visitor:openseasame' | base64
                            "Basic dmlzaXRvcjpvcGVuc2Vhc2FtZQ==",
                        )
                        .call()
                });
                match auth_response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let h = response.header("Content-Type").unwrap_or_default();
                        assert_contains!(h, "text/html");
                        let response_body = response.into_string().unwrap();
                        assert_contains!(
                            response_body,
                            "Welcome to CNB Static Web Server Basic Auth Test!"
                        );
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
            },
        );
        // Check with runtime var disabling auth.
        start_container(
            &ctx,
            ContainerConfig::new()
                .env("WEB_BASIC_AUTH_DISABLED", "true")
                .env("WEB_BASIC_AUTH_USERNAME", "visitor")
                .env(
                    "WEB_BASIC_AUTH_PASSWORD_BCRYPT",
                    // bcrypt password hash generated with
                    //   htpasswd -bnBC 10 "" openseasame | tr -d ':\n'
                    "$2y$10$Uc4licEvYDo2DqtNnTkfV.o0Bvr4Sqw0Vpdh7UMrmkqwTZ92EMkZ6",
                ),
            |container, socket_addr| {
                let response_result = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
                    ureq::get(&format!("http://{socket_addr}")).call()
                });
                match response_result {
                    Ok(response) => {
                        assert_eq!(response.status(), 200);
                        let h = response.header("Content-Type").unwrap_or_default();
                        assert_contains!(h, "text/html");
                        let response_body = response.into_string().unwrap();
                        assert_contains!(
                            response_body,
                            "Welcome to CNB Static Web Server Basic Auth Test!"
                        );
                    }
                    Err(error) => {
                        let logs = container.logs_now();
                        eprint!("Server logs: {logs}");
                        panic!("should respond 200 ok, but got other error: {error:?}");
                    }
                }
            },
        );
    });
}
