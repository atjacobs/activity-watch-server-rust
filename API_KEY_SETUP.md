# API Key Setup - Simple Guide

The server now automatically generates API keys! No scripts needed.

## Quick Start for Remote Access

### Step 1: Configure Remote Access

Create or edit your config file:
- **Windows**: `%APPDATA%\activitywatch\aw-server-rust\config.toml`
- **Linux/Mac**: `~/.config/activitywatch/aw-server-rust/config.toml`

Add:

```toml
address = "0.0.0.0"

[security]
require_auth = true
allow_remote = true
```

**That's it!** No need to manually create API keys.

### Step 2: Start the Server

```bash
aw-server
```

The server will **automatically generate** an API key on first run and display it:

```
==========================================
  AUTO-GENERATED API KEY
==========================================

API Key: AbCdEf123456...

This key has been saved to:
  C:\Users\YourName\AppData\Roaming\activitywatch\aw-server-rust\api_keys.txt

Use this key when connecting to the server:
  Authorization: Bearer AbCdEf123456...

⚠️  IMPORTANT: Save this key now!
==========================================
```

**Save this key!** You'll need it to connect to the server.

### Step 3: Use the API Key

When making requests to the server:

```bash
# Using curl
curl -H "Authorization: Bearer AbCdEf123456..." http://your-server:5600/api/0/info

# Or with X-API-Key header
curl -H "X-API-Key: AbCdEf123456..." http://your-server:5600/api/0/info
```

## Key Locations

The server saves keys in two places:

1. **Plaintext key** (for you to use):
   - Windows: `%APPDATA%\activitywatch\aw-server-rust\api_keys.txt`
   - Linux/Mac: `~/.config/activitywatch/aw-server-rust/api_keys.txt`

2. **Hashed key** (in config.toml):
   - Automatically added to `[security]` section
   - This is what the server uses to verify requests

## Generate Additional Keys

Want to create more API keys?

```bash
# Generate a new key
aw-server --generate-api-key

# It will be added to your config automatically
```

## Testing

### Test Without API Key (Should Fail)

```bash
curl http://127.0.0.1:5600/api/0/info
# Response: 401 Unauthorized ✓
```

### Test With API Key (Should Work)

```bash
curl -H "Authorization: Bearer YOUR_KEY" http://127.0.0.1:5600/api/0/info
# Response: {"hostname": "...", ...} ✓
```

## Local Development (No Auth Needed)

For local-only use, just run the server with default config:

```bash
# No config file needed
aw-server

# Binds to 127.0.0.1:5600
# No authentication required ✓
```

## Troubleshooting

### "No API keys configured"

The server will auto-generate one. Just watch the console output.

### Lost Your API Key?

Generate a new one:

```bash
aw-server --generate-api-key
```

Or check the saved file:
- Windows: `%APPDATA%\activitywatch\aw-server-rust\api_keys.txt`
- Linux/Mac: `~/.config/activitywatch/aw-server-rust/api_keys.txt`

### Can't Access from Another Machine?

1. Check config has `address = "0.0.0.0"`
2. Check firewall allows port 5600
3. Verify you're using the API key in requests

## Security Notes

- API keys are **auto-generated** as 32 random alphanumeric characters
- Keys are **hashed with SHA256** before storage in config
- Plaintext keys are saved in a separate file for your reference
- **Delete `api_keys.txt` after saving the key** somewhere secure
- Use **TLS/HTTPS** for remote access (see REMOTE_ACCESS.md)

## Comparison: Old vs New

### Old Way (Scripts) ❌

```bash
# Generate key manually
./scripts/generate-api-key.sh
# or
.\scripts\generate-api-key.ps1

# Copy hash to config.toml
# Edit config manually
# Restart server
```

### New Way (Automatic) ✅

```bash
# Just start the server
aw-server

# Key is generated and displayed automatically!
```

Much simpler!

## Advanced: Python Client Example

```python
from aw_client import ActivityWatchClient

client = ActivityWatchClient(
    host="https://your-server.com",
    port=5600
)

# Add API key to headers
client.request_headers = {
    "Authorization": "Bearer YOUR_API_KEY"
}

# Use normally
buckets = client.get_buckets()
```

## Summary

1. **Set remote access in config** (`address = "0.0.0.0"`, `allow_remote = true`, `require_auth = true`)
2. **Start server** (`aw-server`)
3. **Copy the displayed API key** (it's auto-generated!)
4. **Use the key** in your client requests

Done! 🎉
