#[macro_use]
extern crate log;

use std::env;
use std::path::PathBuf;

use clap::crate_version;
use clap::Parser;

use aw_server::*;

#[cfg(target_os = "linux")]
use sd_notify::NotifyState;
#[cfg(all(target_os = "linux", target_arch = "x86"))]
extern crate jemallocator;
#[cfg(all(target_os = "linux", target_arch = "x86"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Rust server for ActivityWatch
#[derive(Parser)]
#[clap(version = crate_version!(), author = "Johan Bjäreholt, Erik Bjäreholt, et al.")]
struct Opts {
    /// Run in testing mode
    #[clap(long)]
    testing: bool,

    /// Verbose output
    #[clap(long)]
    verbose: bool,

    /// Address to listen to
    #[clap(long)]
    host: Option<String>,

    /// Port to listen on
    #[clap(long)]
    port: Option<String>,

    /// Path to database override
    /// Also implies --no-legacy-import if no db found
    #[clap(long)]
    dbpath: Option<String>,

    /// Path to webui override
    #[clap(long)]
    webpath: Option<String>,

    /// Mapping of custom static paths to serve, in the format: watcher1=/path,watcher2=/path2
    #[clap(long)]
    custom_static: Option<String>,

    /// Device ID override
    #[clap(long)]
    device_id: Option<String>,

    /// Don't import from aw-server-python if no aw-server-rust db found
    #[clap(long)]
    no_legacy_import: bool,

    /// Generate a new API key and exit
    #[clap(long)]
    generate_api_key: bool,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let opts: Opts = Opts::parse();

    use std::sync::Mutex;

    let mut testing = opts.testing;

    // Always override environment if --testing is specified
    if !testing && cfg!(debug_assertions) {
        testing = true;
    }

    logging::setup_logger("aw-server-rust", testing, opts.verbose)
        .expect("Failed to setup logging");

    if testing {
        info!("Running server in Testing mode");
    }

    // Handle --generate-api-key flag
    if opts.generate_api_key {
        info!("Generating new API key...");
        match config::generate_and_save_api_key(testing) {
            Ok((api_key, hash, key_file)) => {
                println!();
                println!("==========================================");
                println!("  NEW API KEY GENERATED");
                println!("==========================================");
                println!();
                println!("API Key: {}", api_key);
                println!("Hash:    {}", hash);
                println!();
                println!("The plaintext key has been saved to:");
                println!("  {}", key_file.display());
                println!();
                println!("The hash has been added to your config:");
                let config_path = dirs::get_config_dir()
                    .unwrap()
                    .join(if testing { "config-testing.toml" } else { "config.toml" });
                println!("  {}", config_path.display());
                println!();
                println!("Use the API key (not the hash) when connecting:");
                println!("  Authorization: Bearer {}", api_key);
                println!();
                println!("==========================================");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to generate API key: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut config = config::create_config(testing);

    // set host if overridden
    if let Some(host) = opts.host {
        config.address = host;
    }

    // set port if overridden
    if let Some(port) = opts.port {
        config.port = port.parse().unwrap();
    }

    // Security warnings
    let is_remote = config.address != "127.0.0.1" && config.address != "localhost";
    if is_remote {
        warn!("⚠️  Server is binding to a non-localhost address: {}", config.address);

        if !config.security.allow_remote {
            error!("❌ Remote access is not allowed. Set security.allow_remote = true in config.toml");
            error!("   or bind to localhost (127.0.0.1) for local-only access");
            std::process::exit(1);
        }

        if !config.security.require_auth {
            error!("❌ CRITICAL SECURITY WARNING: Remote access enabled WITHOUT authentication!");
            error!("   Set security.require_auth = true and add API keys to config.toml");
            error!("   Or run: aw-server --generate-api-key");
            std::process::exit(1);
        }

        // Auto-generate API key if none exists
        if config.security.api_keys.is_empty() {
            warn!("⚠️  No API keys configured. Generating one automatically...");
            println!();

            match config::generate_and_save_api_key(testing) {
                Ok((api_key, _hash, key_file)) => {
                    info!("✓ API key generated successfully!");
                    println!();
                    println!("==========================================");
                    println!("  AUTO-GENERATED API KEY");
                    println!("==========================================");
                    println!();
                    println!("API Key: {}", api_key);
                    println!();
                    println!("This key has been saved to:");
                    println!("  {}", key_file.display());
                    println!();
                    println!("Use this key when connecting to the server:");
                    println!("  Authorization: Bearer {}", api_key);
                    println!("  or");
                    println!("  X-API-Key: {}", api_key);
                    println!();
                    println!("⚠️  IMPORTANT: Save this key now!");
                    println!("   You'll need it to access the server.");
                    println!();
                    println!("==========================================");
                    println!();

                    // Reload config to get the updated key
                    config = config::create_config(testing);
                }
                Err(e) => {
                    error!("❌ Failed to generate API key: {}", e);
                    error!("   Run manually: aw-server --generate-api-key");
                    std::process::exit(1);
                }
            }
        }

        if !config.tls.enabled {
            warn!("⚠️  WARNING: Remote access enabled without TLS/HTTPS encryption!");
            warn!("   Your data will be transmitted in plain text over the network.");
            warn!("   Enable TLS by setting tls.enabled = true in config.toml");
        }

        info!("✓ Remote access enabled with authentication");
    }

    // set custom_static if overridden, transform into map
    if let Some(custom_static_str) = opts.custom_static {
        let custom_static_map: std::collections::HashMap<String, String> = custom_static_str
            .split(',')
            .map(|s| {
                let mut split = s.split('=');
                let key = split.next().unwrap().to_string();
                let value = split.next().unwrap().to_string();
                (key, value)
            })
            .collect();
        config.custom_static.extend(custom_static_map);

        // validate paths, log error if invalid
        // remove invalid paths
        for (name, path) in config.custom_static.clone().iter() {
            if !std::path::Path::new(path).exists() {
                error!("custom_static path for {} does not exist ({})", name, path);
                config.custom_static.remove(name);
            }
        }
    }

    // Set db path if overridden
    let db_path: String = if let Some(dbpath) = opts.dbpath.clone() {
        dbpath
    } else {
        dirs::db_path(testing)
            .expect("Failed to get db path")
            .to_str()
            .unwrap()
            .to_string()
    };
    info!("Using DB at path {:?}", db_path);

    let asset_path = opts.webpath.map(|webpath| PathBuf::from(webpath));
    info!("Using aw-webui assets at path {:?}", asset_path);

    // Only use legacy import if opts.dbpath is not set
    let legacy_import = !opts.no_legacy_import && opts.dbpath.is_none();
    if opts.dbpath.is_some() {
        info!("Since custom dbpath is set, --no-legacy-import is implied");
    }

    let device_id: String = if let Some(id) = opts.device_id {
        id
    } else {
        device_id::get_device_id()
    };

    let server_state = endpoints::ServerState {
        // Even if legacy_import is set to true it is disabled on Android so
        // it will not happen there
        datastore: Mutex::new(aw_datastore::Datastore::new(db_path, legacy_import)),
        asset_resolver: endpoints::AssetResolver::new(asset_path),
        device_id,
    };

    let _rocket = endpoints::build_rocket(server_state, config)
        .ignite()
        .await?;
    #[cfg(target_os = "linux")]
    let _ = sd_notify::notify(true, &[NotifyState::Ready]);
    _rocket.launch().await?;

    Ok(())
}
