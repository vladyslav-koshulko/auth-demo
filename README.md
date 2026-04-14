# Demo OAuth 2.0 / OIDC CLI Application

A production-ready demonstration of OAuth 2.0 and OpenID Connect authentication in a Rust CLI application with Google as the identity provider.

## Overview

This project demonstrates a secure OAuth 2.0 + OpenID Connect implementation with the following features:
- Command-line interface for authentication
- Secure session management with encryption
- Automatic token refresh
- Web callback server for OAuth flow
- Protected API endpoints

## Technologies Used

- **Rust 2024 Edition**
- **Tokio** - Async runtime
- **Axum** - Web framework for callback server
- **Clap** - CLI argument parsing
- **jsonwebtoken** - JWT validation
- **aes-gcm** - Session encryption (AES-256-GCM)
- **pbkdf2** - Key derivation for encryption
- **reqwest** - HTTP client for token exchange

## Implemented Features

### Security (Production-Ready)
- ✅ **PKCE S256** - Proof Key for Code Exchange (OAuth 2.1 requirement)
- ✅ **Nonce validation** - Protection against replay attacks
- ✅ **State parameter** - CSRF protection
- ✅ **JWT validation** - Signature, audience, issuer, expiration
- ✅ **JWKS caching** - Public key caching with TTL
- ✅ **Session encryption** - AES-256-GCM with PBKDF2 key derivation

### Session Management
- ✅ **Encrypted file storage** - Sessions stored in `.session_db` (encrypted)
- ✅ **Automatic token refresh** - Refreshes access tokens before expiration
- ✅ **Session cleanup** - CLI command to remove expired sessions
- ✅ **Graceful shutdown** - Proper signal handling (Ctrl+C, SIGTERM)

### API & Middleware
- ✅ **Auth middleware** - Protects API endpoints (`/me`)
- ✅ **Web callback server** - Handles OAuth callback on `127.0.0.1:8081`
- ✅ **Protected endpoints** - Demo endpoint showing authenticated user info

### CLI Commands
- ✅ `login` - Initiates OAuth flow, opens browser
- ✅ `me` - Shows current user info with auto token refresh
- ✅ `logout` - Clears current session
- ✅ `cleanup` - Removes expired sessions from database

## Not Implemented (Out of Scope for Demo)

- ❌ **Rate limiting** - Would require additional dependencies
- ❌ **Multiple identity providers** - Currently only Google
- ❌ **Web-based session management** - CLI-focused demo
- ❌ **Unit/Integration tests** - Would expand project significantly
- ❌ **Database backend** - Uses file-based storage
- ❌ **Session revocation on provider side** - Would require additional API calls

## Project Structure

```
src/
├── cli.rs           # CLI commands definition
├── main.rs          # Entry point
├── server.rs        # Axum web server (callback + protected routes)
├── middleware/
│   ├── auth.rs      # Auth middleware for protected endpoints
│   └── mod.rs
├── models/
│   └── user.rs      # User struct
├── oauth/
│   ├── claims.rs    # JWT claims structures
│   ├── client.rs    # Token exchange & refresh
│   ├── google.rs    # Google OAuth URLs & JWKS
│   ├── jwt.rs       # JWT validation logic
│   ├── jwks.rs      # JWKS data structures
│   └── jwks_cache.rs # In-memory JWKS cache
├── session/
│   └── file.rs      # Encrypted file-based session storage
└── utils/
    ├── crypto.rs    # Random string generation & PKCE
    └── session_encryption.rs # AES-256-GCM encryption
```

## Setup

### 1. Google Cloud Console Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Navigate to "APIs & Services" → "Credentials"
4. Click "Create Credentials" → "OAuth client ID"
5. Select "Desktop app" as application type
6. Note your **Client ID** and **Client Secret**
7. Add `http://127.0.0.1:8081/callback` to authorized redirect URIs

### 2. Environment Configuration

Create a `.env` file in the project root:

```env
# Google OAuth credentials
GOOGLE_CLIENT_ID=your_client_id_here
GOOGLE_CLIENT_SECRET=your_client_secret_here

# OAuth endpoints
AUTH_URI=https://accounts.google.com/o/oauth2/v2/auth
CERT_URI=https://www.googleapis.com/oauth2/v3/certs
REDIRECT_URL=http://127.0.0.1:8081/callback

# Session encryption key (generate a strong random string)
SESSION_KEY=your_random_encryption_key_min_16_chars
```

**Important:** Generate a strong `SESSION_KEY` for encrypting sessions:
```bash
openssl rand -base64 32
```

### 3. Build & Run

```bash
# Build the project
cargo build --release

# Or run directly
cargo run -- <command>
```

## Usage

### Login

```bash
cargo run -- login
```

This will:
1. Generate PKCE parameters (code_verifier, code_challenge)
2. Generate state and nonce for security
3. Open your browser to Google authorization page
4. Start a local web server on `127.0.0.1:8081`
5. Wait for callback with authorization code
6. Exchange code for tokens (access_token, id_token, refresh_token)
7. Validate ID token (signature, nonce, audience, issuer)
8. Create encrypted session and save to `.session_db`

### Check Current User

```bash
cargo run -- me
```

Shows:
- User ID (sub from ID token)
- Name
- Email
- Token expiration time

Automatically refreshes access token if expired.

### Logout

```bash
cargo run -- logout
```

Clears the current session reference (does not delete from database).

### Cleanup Expired Sessions

```bash
cargo run -- cleanup
```

Removes all expired sessions from the encrypted database.

## Security Considerations

### Implemented
- **PKCE S256**: Prevents authorization code interception attacks
- **Nonce**: Prevents replay attacks
- **State**: Prevents CSRF attacks
- **Token binding**: Sessions are cryptographically bound to tokens
- **Encryption at rest**: Session database is encrypted with AES-256-GCM
- **Key derivation**: Uses PBKDF2 with 100k iterations
- **No plaintext secrets**: All sensitive data encrypted or in environment

### For Production Deployment
- Use a proper database (PostgreSQL, Redis) instead of file storage
- Add rate limiting at load balancer/reverse proxy level
- Implement session revocation on identity provider logout
- Use HTTPS for all endpoints (not localhost)
- Add structured logging and monitoring
- Implement audit logging for authentication events

## Testing Protected Endpoint

After login, you can test the protected web endpoint:

```bash
# In one terminal, start a long-running process
cargo run -- login

# In another terminal, after successful login
curl http://127.0.0.1:8081/me
```

You should see your authenticated user info.

## Troubleshooting

### "SESSION_KEY not set"
- Ensure `.env` file exists and contains `SESSION_KEY`
- Key should be at least 16 characters

### "Invalid state" error
- This means the state parameter doesn't match
- Can happen if you restart the server mid-flow
- Just run `login` again

### "Nonce validation failed"
- ID token nonce doesn't match expected value
- Can happen if you reuse an old authorization URL
- Each login generates fresh nonce

### "Failed to refresh token"
- Refresh token may be expired or revoked
- Run `logout` then `login` again

## License

MIT License - This is a demonstration project.

## Contributing

This is a demo project for educational purposes. Feel free to fork and extend.
