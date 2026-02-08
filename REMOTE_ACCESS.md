# Remote Access and Security Guide

This guide explains how to configure ActivityWatch server for secure remote access with API key authentication and TLS encryption.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Security Features](#security-features)
- [Configuration](#configuration)
- [Generating API Keys](#generating-api-keys)
- [Setting Up TLS/HTTPS](#setting-up-tlshttps)
- [Client Configuration](#client-configuration)
- [Troubleshooting](#troubleshooting)

## Overview

By default, ActivityWatch server only accepts connections from localhost (127.0.0.1) for security reasons. This update adds:

- **API Key Authentication**: Protect your server with API keys
- **TLS/HTTPS Support**: Encrypt data in transit
- **Remote Access Controls**: Configure who can access your server
- **Security Warnings**: Prevent insecure configurations

## Quick Start

### 1. Generate an API Key

```bash
cd aw-server-rust
./scripts/generate-api-key.sh
```

This will output:
- A random API key (save this!)
- The SHA256 hash to add to your config
- Configuration instructions

### 2. Update Configuration

Edit your config file:
- **Linux**: `~/.config/activitywatch/aw-server-rust/config.toml`
- **macOS**: `~/Library/Application Support/activitywatch/aw-server-rust/config.toml`
- **Windows**: `%APPDATA%\activitywatch\aw-server-rust\config.toml`

Add:

```toml
# Allow connections from any network interface
address = "0.0.0.0"

[security]
# Require API key authentication
require_auth = true

# Allow remote connections
allow_remote = true

# API key hashes (from generate-api-key.sh)
api_keys = [
    "your-hash-here",
]

[tls]
# Enable HTTPS (recommended for remote access)
enabled = false  # Set to true when you have certificates
cert = "/path/to/cert.pem"
key = "/path/to/key.pem"
```

### 3. Restart the Server

The server will now require authentication for all API endpoints.

## Security Features

### API Key Authentication

API keys are hashed using SHA256 before storage. Clients must provide the plaintext key, which is hashed and compared against stored hashes.

**Supported header formats:**
```
Authorization: Bearer <api-key>
X-API-Key: <api-key>
```

### TLS/HTTPS Support

When enabled, all traffic is encrypted using TLS. This prevents eavesdropping and man-in-the-middle attacks.

### Host Header Validation

When bound to localhost, the server validates the Host header to prevent DNS rebinding attacks.

### Security Checks

The server performs validation on startup:
- ❌ **Blocks** remote access without authentication
- ⚠️  **Warns** about remote access without TLS
- ✓ **Allows** properly secured remote configurations

## Configuration

### Security Section

```toml
[security]
# Require API key for all API endpoints
require_auth = true  # Default: false

# Allow binding to non-localhost addresses
allow_remote = true  # Default: false

# List of API key hashes (SHA256)
api_keys = [
    "hash1",
    "hash2",
]
```

### TLS Section

```toml
[tls]
# Enable TLS/HTTPS
enabled = true  # Default: false

# Path to certificate file
cert = "/path/to/cert.pem"

# Path to private key file
key = "/path/to/key.pem"
```

## Generating API Keys

### Using the Script

```bash
./scripts/generate-api-key.sh
```

This generates a random 32-character API key and its hash.

### Using a Custom Key

```bash
./scripts/generate-api-key.sh "my-secret-key"
```

### Manual Generation

```bash
# Generate hash
echo -n "my-secret-key" | sha256sum

# Add the hash (first part) to config.toml
```

## Setting Up TLS/HTTPS

### Self-Signed Certificate (for testing/private networks)

```bash
# Generate certificate and key
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout key.pem -out cert.pem -days 365 \
  -subj "/CN=your-hostname"

# Update config.toml
[tls]
enabled = true
cert = "/path/to/cert.pem"
key = "/path/to/key.pem"
```

**Note**: Clients will need to accept the self-signed certificate.

### Let's Encrypt Certificate (for public servers)

```bash
# Install certbot
sudo apt-get install certbot  # Debian/Ubuntu

# Get certificate
sudo certbot certonly --standalone -d your-domain.com

# Update config.toml
[tls]
enabled = true
cert = "/etc/letsencrypt/live/your-domain.com/fullchain.pem"
key = "/etc/letsencrypt/live/your-domain.com/privkey.pem"
```

### Automatic Renewal

Add a renewal hook to reload the server:

```bash
# /etc/letsencrypt/renewal-hooks/post/reload-aw-server.sh
#!/bin/bash
systemctl reload aw-server
```

## Client Configuration

### Python Client

```python
from aw_client import ActivityWatchClient

client = ActivityWatchClient(
    host="https://your-server.com",
    port=5600
)

# Set API key
client.request_headers = {
    "Authorization": "Bearer your-api-key-here"
}

# Use normally
buckets = client.get_buckets()
```

### cURL

```bash
# With Bearer token
curl -H "Authorization: Bearer your-api-key" \
  https://your-server.com:5600/api/0/info

# With X-API-Key header
curl -H "X-API-Key: your-api-key" \
  https://your-server.com:5600/api/0/info
```

### Browser Extension

Update the server URL in extension settings:
```
https://your-server.com:5600
```

Add API key in custom headers (if supported by extension).

## Troubleshooting

### "Remote access is not allowed"

**Error**: Server exits with "Remote access is not allowed"

**Solution**: Set `security.allow_remote = true` in config.toml

### "Authentication required"

**Error**: 401 Unauthorized responses

**Solutions**:
1. Check API key is correct
2. Verify hash in config matches: `echo -n "your-key" | sha256sum`
3. Check header format: `Authorization: Bearer <key>`

### "Host header is invalid"

**Error**: 400 Bad Request with host header error

**Solution**: When accessing remotely, ensure:
- `address` is not "127.0.0.1" or "localhost"
- Or use `--host 0.0.0.0` flag when starting server

### TLS Certificate Errors

**Error**: Certificate verification failed

**Solutions**:
- For self-signed certs: Accept/trust the certificate
- For Let's Encrypt: Ensure certificate is not expired
- Check cert/key paths in config are correct
- Verify certificate permissions (readable by server)

### WebUI Accessible but API Blocked

The WebUI (HTML/CSS/JS files) is always accessible without authentication. Only API endpoints require authentication. This is by design to allow users to view the interface and troubleshoot.

## Security Best Practices

1. **Always use TLS** for remote access
2. **Use strong API keys** (32+ random characters)
3. **Rotate keys regularly** (generate new keys periodically)
4. **Limit network access** with firewall rules
5. **Monitor access logs** for suspicious activity
6. **Keep server updated** with security patches
7. **Use separate keys** for different clients/users

## Migration from Local-Only Setup

If you're upgrading from a local-only installation:

1. Existing local clients will continue working (auth not required for localhost)
2. Remote clients will need API keys
3. Config files are backward compatible (new sections are optional)
4. Default behavior unchanged (localhost only, no auth)

## Technical Details

### Authentication Flow

1. Client sends request with API key in header
2. Server extracts key from `Authorization` or `X-API-Key` header
3. Server hashes key using SHA256
4. Server compares hash against configured hashes
5. If match found, request proceeds; otherwise 401 returned

### Endpoints Protected

All `/api/*` endpoints require authentication when enabled:
- `/api/0/info`
- `/api/0/buckets/*`
- `/api/0/query`
- `/api/0/import`
- `/api/0/export`
- `/api/0/settings`

### Endpoints Always Public

These endpoints never require authentication:
- `/` (WebUI)
- `/css/*`, `/js/*`, `/fonts/*`, `/static/*`
- `/favicon.ico`, `/logo.png`, `/manifest.json`

This allows the WebUI to load and display login/configuration instructions.

## Contributing

Found a security issue? Please report it privately to the ActivityWatch security team.

## License

Same as ActivityWatch main project (MPL-2.0).
