//! API Key Authentication
//!
//! This module provides authentication for remote access to ActivityWatch server.
//! API keys are hashed using SHA256 before storage in the config file.
//!
//! To generate an API key hash:
//! ```bash
//! echo -n "your-secret-api-key" | sha256sum
//! ```

use rocket::fairing::Fairing;
use rocket::http::uri::Origin;
use rocket::http::{Method, Status};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::route;
use rocket::{Data, Rocket, Route, State};
use sha2::{Digest, Sha256};

use crate::config::{AWConfig, SecurityConfig};
use crate::endpoints::HttpErrorJson;

static FAIRING_ROUTE_BASE: &str = "/auth_fairing";

/// Authentication Fairing
/// Intercepts all requests and validates API key if authentication is required
pub struct AuthCheck {
    config: SecurityConfig,
    is_localhost: bool,
}

impl AuthCheck {
    pub fn new(config: &AWConfig) -> AuthCheck {
        let is_localhost = config.address == "127.0.0.1" || config.address == "localhost";
        AuthCheck {
            config: config.security.clone(),
            is_localhost,
        }
    }

    /// Hash an API key using SHA256
    pub fn hash_api_key(api_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify an API key against stored hashes
    fn verify_api_key(&self, api_key: &str) -> bool {
        if self.config.api_keys.is_empty() {
            warn!("No API keys configured but authentication is required!");
            return false;
        }

        let hash = Self::hash_api_key(api_key);
        self.config.api_keys.contains(&hash)
    }

    /// Extract and validate API key from request
    fn validate_request(&self, request: &Request) -> bool {
        // Check Authorization header
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            // Support "Bearer <token>" format
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                return self.verify_api_key(token);
            }
            // Support direct token in Authorization header
            return self.verify_api_key(auth_header);
        }

        // Check X-API-Key header (alternative format)
        if let Some(api_key) = request.headers().get_one("X-API-Key") {
            return self.verify_api_key(api_key);
        }

        false
    }
}

/// Create a `Handler` for authentication error handling
#[derive(Clone)]
struct AuthErrorRoute {}

#[rocket::async_trait]
impl rocket::route::Handler for AuthErrorRoute {
    async fn handle<'r>(
        &self,
        request: &'r Request<'_>,
        _: rocket::Data<'r>,
    ) -> rocket::route::Outcome<'r> {
        let err = HttpErrorJson::new(
            Status::Unauthorized,
            "Authentication required. Provide API key via 'Authorization: Bearer <key>' or 'X-API-Key' header".to_string(),
        );
        route::Outcome::from(request, err)
    }
}

/// Create a new `Route` for authentication error handling
fn auth_error_route() -> Route {
    Route::ranked(1, Method::Get, "/", AuthErrorRoute {})
}

fn redirect_unauthorized(request: &mut Request) {
    let uri = FAIRING_ROUTE_BASE.to_string();
    let origin = Origin::parse_owned(uri).unwrap();
    request.set_method(Method::Get);
    request.set_uri(origin);
}

#[rocket::async_trait]
impl Fairing for AuthCheck {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "AuthCheck",
            kind: rocket::fairing::Kind::Ignite | rocket::fairing::Kind::Request,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<rocket::Build>) -> rocket::fairing::Result {
        if self.config.require_auth {
            info!("API key authentication is ENABLED for remote clients");
            info!("Local clients (127.0.0.1/localhost) can connect without authentication");
            if self.config.api_keys.is_empty() {
                error!("Authentication is required but no API keys are configured!");
                error!("Add API key hashes to the [security] section of your config.toml");
                error!("Generate a hash with: echo -n 'your-key' | sha256sum");
            } else {
                info!("Configured with {} API key(s)", self.config.api_keys.len());
            }
            Ok(rocket.mount(FAIRING_ROUTE_BASE, vec![auth_error_route()]))
        } else {
            info!("API key authentication is DISABLED");
            if !self.is_localhost {
                warn!("WARNING: Remote access enabled without authentication - this is a security risk!");
            }
            Ok(rocket)
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        // Check if the CLIENT is connecting from localhost
        // This is key for backward compatibility - local clients should always work
        let client_is_localhost = if let Some(remote_addr) = request.remote() {
            let ip = remote_addr.ip();
            ip.is_loopback()
        } else {
            false
        };

        // ALWAYS allow localhost clients without authentication
        // This ensures backward compatibility with aw-qt and other local clients
        if client_is_localhost {
            return;
        }

        // For remote connections, check if auth is required
        if !self.config.require_auth {
            // Authentication is disabled
            return;
        }

        // Allow requests to the webui and static files without authentication
        let path = request.uri().path();
        if path.starts_with("/css")
            || path.starts_with("/js")
            || path.starts_with("/fonts")
            || path.starts_with("/static")
            || path == "/"
            || path == "/favicon.ico"
            || path == "/logo.png"
            || path == "/manifest.json"
            || path == "/dark.css"
        {
            return;
        }

        // Validate API key for all API endpoints
        if !self.validate_request(request) {
            info!("Unauthorized request to {}, denying", path);
            redirect_unauthorized(request);
        }
    }
}

/// Request Guard for API authentication
/// Use this to protect specific endpoints that require authentication
pub struct ApiKey;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = request
            .guard::<&State<AWConfig>>()
            .await
            .expect("AWConfig not found in state");

        // Check if the CLIENT is connecting from localhost
        let client_is_localhost = if let Some(remote_addr) = request.remote() {
            let ip = remote_addr.ip();
            ip.is_loopback()
        } else {
            false
        };

        // ALWAYS allow localhost clients without authentication
        if client_is_localhost {
            return Outcome::Success(ApiKey);
        }

        // If auth is not required, allow the request
        if !config.security.require_auth {
            return Outcome::Success(ApiKey);
        }

        // Check Authorization header
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            let token = auth_header
                .strip_prefix("Bearer ")
                .unwrap_or(auth_header);

            let hash = AuthCheck::hash_api_key(token);
            if config.security.api_keys.contains(&hash) {
                return Outcome::Success(ApiKey);
            }
        }

        // Check X-API-Key header
        if let Some(api_key) = request.headers().get_one("X-API-Key") {
            let hash = AuthCheck::hash_api_key(api_key);
            if config.security.api_keys.contains(&hash) {
                return Outcome::Success(ApiKey);
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_api_key() {
        let key = "test-secret-key";
        let hash = AuthCheck::hash_api_key(key);

        // SHA256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);

        // Same key should produce same hash
        assert_eq!(hash, AuthCheck::hash_api_key(key));

        // Different key should produce different hash
        assert_ne!(hash, AuthCheck::hash_api_key("different-key"));
    }

    #[test]
    fn test_verify_api_key() {
        let mut config = SecurityConfig::default();
        let key = "my-secret-key";
        let hash = AuthCheck::hash_api_key(key);
        config.api_keys.push(hash);

        let auth_check = AuthCheck {
            config,
            is_localhost: false,
        };

        // Correct key should verify
        assert!(auth_check.verify_api_key(key));

        // Wrong key should not verify
        assert!(!auth_check.verify_api_key("wrong-key"));
    }
}
