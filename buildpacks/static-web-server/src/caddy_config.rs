use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyConfig {
    pub(crate) apps: CaddyConfigApps,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyConfigApps {
    pub(crate) http: CaddyConfigAppHTTP,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyConfigAppHTTP {
    pub(crate) servers: CaddyConfigHTTPServers,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyConfigHTTPServers {
    pub(crate) public: CaddyConfigHTTPServerPublic,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyConfigHTTPServerPublic {
    pub(crate) listen: Vec<String>,
    pub(crate) routes: Vec<CaddyHTTPServerRoute>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyHTTPServerRoute {
    pub(crate) r#match: Option<Vec<CaddyHTTPServerRouteMatcher>>,
    pub(crate) handle: Vec<CaddyHTTPServerRouteHandler>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CaddyHTTPServerErrors {
    pub(crate) routes: Vec<CaddyHTTPServerRoute>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum CaddyHTTPServerRouteMatcher {
    // https://caddyserver.com/docs/json/apps/http/servers/routes/match/path/
    Path(MatchPath),
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/match/path/
#[derive(Serialize, Deserialize)]
pub(crate) struct MatchPath {
    pub(crate) path: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum CaddyHTTPServerRouteHandler {
    FileServer(FileServer),
    Headers(Headers),
    StaticResponse(StaticResponse),
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/file_server/
#[derive(Serialize, Deserialize)]
pub(crate) struct FileServer {
    pub(crate) handler: String,
    pub(crate) root: String,
    pub(crate) pass_thru: bool,
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/headers/
#[derive(Serialize, Deserialize)]
pub(crate) struct Headers {
    pub(crate) handler: String,
    pub(crate) response: HeadersResponse,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HeadersResponse {
    pub(crate) set: serde_json::Map<String, serde_json::Value>,
    pub(crate) deferred: bool,
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/static_response/
#[derive(Serialize, Deserialize)]
pub(crate) struct StaticResponse {
    pub(crate) handler: String,
    pub(crate) status_code: String,
    pub(crate) headers: Option<serde_json::Map<String, serde_json::Value>>,
    pub(crate) body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "currently failing"]
    fn parse_simple_caddy_config() {
        let config = r#"
            {
                "apps": {
                    "http": {
                        "servers": {
                            "public": {
                                "listen": [":{env.PORT}"],
                                "routes": [
                                    {
                                        "handle": [
                                            {
                                                "handler": "file_server",
                                                "root": "public/"
                                            }
                                        ]
                                    }
                                ]
                            }
                        }
                    }
                }
            }
        "#;

        let parsed_config = serde_json::from_str::<CaddyConfig>(config).unwrap();

        // TODO: Additional asserts where necessary
        assert_eq!(
            parsed_config.apps.http.servers.public.listen,
            vec![String::from(":{env.PORT}")]
        );
    }
}
