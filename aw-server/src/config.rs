use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use rocket::config::Config;
use rocket::data::{Limits, ToByteUnit};
use rocket::log::LogLevel;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::dirs;

// Far from an optimal way to solve it, but works and is simple
static mut TESTING: bool = true;
pub fn set_testing(testing: bool) {
    unsafe {
        TESTING = testing;
    }
}
pub fn is_testing() -> bool {
    unsafe { TESTING }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "default_require_auth")]
    pub require_auth: bool,

    #[serde(default = "default_api_keys")]
    pub api_keys: Vec<String>, // Stored as SHA256 hashes

    #[serde(default = "default_allow_remote")]
    pub allow_remote: bool,
}

impl Default for SecurityConfig {
    fn default() -> SecurityConfig {
        SecurityConfig {
            require_auth: default_require_auth(),
            api_keys: default_api_keys(),
            allow_remote: default_allow_remote(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TlsConfig {
    #[serde(default = "default_tls_enabled")]
    pub enabled: bool,

    #[serde(default = "default_tls_cert")]
    pub cert: String,

    #[serde(default = "default_tls_key")]
    pub key: String,
}

impl Default for TlsConfig {
    fn default() -> TlsConfig {
        TlsConfig {
            enabled: default_tls_enabled(),
            cert: default_tls_cert(),
            key: default_tls_key(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AWConfig {
    #[serde(default = "default_address")]
    pub address: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(skip, default = "default_testing")]
    pub testing: bool, // This is not written to the config file (serde(skip))

    #[serde(default = "default_cors")]
    pub cors: Vec<String>,

    // A mapping of watcher names to paths where the
    // custom visualizations are located.
    #[serde(default = "default_custom_static")]
    pub custom_static: std::collections::HashMap<String, String>,

    #[serde(default)]
    pub security: SecurityConfig,

    #[serde(default)]
    pub tls: TlsConfig,
}

impl Default for AWConfig {
    fn default() -> AWConfig {
        AWConfig {
            address: default_address(),
            port: default_port(),
            testing: default_testing(),
            cors: default_cors(),
            custom_static: default_custom_static(),
            security: SecurityConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl AWConfig {
    pub fn to_rocket_config(&self) -> rocket::Config {
        let mut config;
        if self.testing {
            config = Config::debug_default();
            config.log_level = LogLevel::Debug;
        } else {
            config = Config::release_default()
        };

        // Needed for bucket imports
        let limits = Limits::default()
            .limit("json", 1000u64.megabytes())
            .limit("data-form", 1000u64.megabytes());

        config.address = self.address.parse().unwrap();
        config.port = self.port;
        config.keep_alive = 0;
        config.limits = limits;

        // Configure TLS if enabled
        if self.tls.enabled {
            use std::path::PathBuf;
            config.tls = Some(rocket::config::TlsConfig::from_paths(
                PathBuf::from(&self.tls.cert),
                PathBuf::from(&self.tls.key),
            ));
        }

        config
    }
}

fn default_address() -> String {
    "127.0.0.1".to_string()
}

fn default_cors() -> Vec<String> {
    Vec::<String>::new()
}

fn default_testing() -> bool {
    is_testing()
}

fn default_port() -> u16 {
    if is_testing() {
        5666
    } else {
        5600
    }
}

fn default_custom_static() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}

fn default_require_auth() -> bool {
    false
}

fn default_api_keys() -> Vec<String> {
    Vec::new()
}

fn default_allow_remote() -> bool {
    false
}

fn default_tls_enabled() -> bool {
    false
}

fn default_tls_cert() -> String {
    "cert.pem".to_string()
}

fn default_tls_key() -> String {
    "key.pem".to_string()
}

pub fn create_config(testing: bool) -> AWConfig {
    set_testing(testing);
    let mut config_path = dirs::get_config_dir().unwrap();
    if !testing {
        config_path.push("config.toml")
    } else {
        config_path.push("config-testing.toml")
    }

    /* If there is no config file, create a new config file with default values but every value is
     * commented out by default in case we would change a default value at some point in the future */
    if !config_path.is_file() {
        debug!("Writing default commented out config at {:?}", config_path);
        let mut wfile = File::create(config_path.clone()).expect("Unable to create config file");
        let default_config = AWConfig::default();
        let default_config_str =
            toml::to_string(&default_config).expect("Failed to convert default config to string");
        let mut default_config_str_commented = String::new();
        default_config_str_commented.push_str("### DEFAULT SETTINGS ###\n");
        for line in default_config_str.lines() {
            default_config_str_commented.push_str(&format!("#{line}\n"));
        }
        wfile
            .write_all(&default_config_str_commented.into_bytes())
            .expect("Failed to write config to file");
        wfile.sync_all().expect("Unable to sync config file");
    }

    debug!("Reading config at {:?}", config_path);
    let mut rfile = File::open(config_path).expect("Failed to open config file for reading");
    let mut content = String::new();
    rfile
        .read_to_string(&mut content)
        .expect("Failed to read config as a string");
    let aw_config: AWConfig = toml::from_str(&content).expect("Failed to parse config file");

    aw_config
}

/// Generate a random API key (32 alphanumeric characters)
pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Hash an API key using SHA256
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Get the path where API keys are stored
pub fn get_api_key_file(testing: bool) -> PathBuf {
    let mut path = dirs::get_config_dir().unwrap();
    if testing {
        path.push("api_keys_testing.txt");
    } else {
        path.push("api_keys.txt");
    }
    path
}

/// Save an API key (plaintext) to a file for user reference
pub fn save_api_key_to_file(api_key: &str, testing: bool) -> Result<PathBuf, std::io::Error> {
    let key_file = get_api_key_file(testing);
    let mut file = File::create(&key_file)?;
    writeln!(file, "# ActivityWatch API Keys")?;
    writeln!(file, "# Keep this file secure!")?;
    writeln!(file, "#")?;
    writeln!(file, "# Use this key when connecting to the server:")?;
    writeln!(file, "#   Authorization: Bearer <key>")?;
    writeln!(file, "#   or")?;
    writeln!(file, "#   X-API-Key: <key>")?;
    writeln!(file, "")?;
    writeln!(file, "{}", api_key)?;
    file.sync_all()?;
    Ok(key_file)
}

/// Generate a new API key, add it to config, and save for user reference
pub fn generate_and_save_api_key(testing: bool) -> Result<(String, String, PathBuf), Box<dyn std::error::Error>> {
    let api_key = generate_api_key();
    let hash = hash_api_key(&api_key);

    // Save the plaintext key for user
    let key_file = save_api_key_to_file(&api_key, testing)?;

    // Read existing config
    let mut config_path = dirs::get_config_dir().unwrap();
    if testing {
        config_path.push("config-testing.toml");
    } else {
        config_path.push("config.toml");
    }

    // Update config with the hash
    let mut config: AWConfig = if config_path.exists() {
        let mut rfile = File::open(&config_path)?;
        let mut content = String::new();
        rfile.read_to_string(&mut content)?;
        toml::from_str(&content)?
    } else {
        AWConfig::default()
    };

    config.security.api_keys.push(hash.clone());

    // Write updated config
    let mut wfile = File::create(&config_path)?;
    let config_str = toml::to_string_pretty(&config)?;
    wfile.write_all(config_str.as_bytes())?;
    wfile.sync_all()?;

    Ok((api_key, hash, key_file))
}
