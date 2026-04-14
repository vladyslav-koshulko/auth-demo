# Полное руководство по внедрению Production Authentication

## Для кого это руководство

Вы Middle/Senior разработчик, который нуждается в структурированном плане внедрения аутентификации и авторизации. Это пошаговый гайд от концепций до production-ready кода.

---

# Часть 1: Концепции и принципы

## 1.1 Что такое аутентификация и авторизация

**Аутентификация (Authentication)** - процесс проверки личности пользователя.
- Подтверждение "ты тот, за кого себя выдаёшь"
- Реализуется через: пароли, JWT токены, OAuth провайдеры

**Авторизация (Authorization)** - процесс проверки прав доступа.
- Решение "что тебе разрешено делать"
- Реализуется через: роли (RBAC), права (permissions), политики (ABAC)

## 1.2 Ключевые принципы безопасности

### Stateless Authentication
- Сервер не хранит состояние сессии
- Вся информация в токене (JWT)
- Масштабируемость: любой сервер может проверить токен

### Separation of Concerns
- Access Token: короткий (15 мин), для доступа к API
- Refresh Token: длинный (7-30 дней), для получения нового Access Token
- Почему: если Access Token украдут, у ворота 15 минут. Refresh Token хранится безопаснее.

### Defense in Depth (Защита в глубину)
- Несколько слоёв защиты
- Если один слой упал, есть другие
- Пример: Rate Limit → Auth Middleware → Permission Check

### Principle of Least Privilege
- Пользователь получает минимум необходимых прав
- Пример: обычный юзер не может удалять других

## 1.3 Технологии и стандарты

### JWT (JSON Web Tokens)
- Стандарт RFC 7519
- Структура: Header.Payload.Signature
- Алгоритмы: RS256 (асимметричный), HS256 (симметричный)

### OAuth 2.0
- Стандарт авторизации (не аутентификации!)
- Flows: Authorization Code, PKCE, Client Credentials, Device Code
- Используется для: "Войти через Google/GitHub"

### OIDC (OpenID Connect)
- Надстройка над OAuth 2.0 для аутентификации
- Возвращает ID Token (JWT с информацией о пользователе)
- Провайдеры: Google, Auth0, Keycloak

### Argon2id
- Современный алгоритм хеширования паролей
- Замена устаревшему bcrypt
- Регулируемая сложность (память + время)

### Cookies безопасность
- HttpOnly: JavaScript не может прочитать
- Secure: только HTTPS
- SameSite: защита от CSRF
- Max-Age: время жизни

---

# Часть 2: Чек-лист внедрения

## Phase 0: Подготовка (День 1)
- [ ] Выбрать провайдера OAuth (Google/Auth0/свой)
- [ ] Настроить CORS policy
- [ ] Подготовить SSL сертификаты (HTTPS)
- [ ] Выбрать БД (Postgres рекомендуется)

## Phase 1: База данных (День 1-2)
- [ ] Создать таблицу users
- [ ] Создать таблицу refresh_tokens
- [ ] Создать таблицу audit_logs (опционально)
- [ ] Настроить индексы

## Phase 2: Ядро аутентификации (День 2-3)
- [ ] Реализовать регистрацию с валидацией
- [ ] Реализовать вход с Argon2id
- [ ] Генерация JWT access токенов
- [ ] Генерация refresh токенов
- [ ] Middleware для проверки JWT

## Phase 3: Управление сессиями (День 3-4)
- [ ] Реализовать refresh endpoint
- [ ] Ротация refresh токенов
- [ ] Logout с удалением из БД
- [ ] Cleanup expired токенов

## Phase 4: Безопасность (День 4-5)
- [ ] Rate limiting на auth endpoints
- [ ] Security headers
- [ ] Audit logging
- [ ] HTTPS enforcement

## Phase 5: OAuth интеграция (День 5-7)
- [ ] Google OAuth setup
- [ ] GitHub OAuth setup
- [ ] Объединение OAuth и локальной auth
- [ ] Обработка ошибок OAuth

## Phase 6: Авторизация (День 7-8)
- [ ] Система ролей (RBAC)
- [ ] Permission-based access
- [ ] Resource-level permissions (владелец может редактировать)

## Phase 7: Production (День 8-10)
- [ ] Graceful shutdown
- [ ] Structured logging
- [ ] Мониторинг (failed login attempts)
- [ ] Тестирование (happy path + edge cases)

---

# Часть 3: Зависимости

## Rust (Cargo.toml)
```toml
[dependencies]
# Web framework
tokio = { version = "1.42", features = ["full"] }
axum = { version = "0.7", features = ["macros"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Auth & Security
jsonwebtoken = "9"
argon2 = "0.5"
rand = "0.8"
secrecy = { version = "0.8", features = ["serde"] }

# Password hashing
password-hash = "0.5"

# Database
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "migrate", "uuid", "chrono"] }
redis = { version = "0.25", features = ["tokio-comp"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }
regex = "1"

# Time
chrono = { version = "0.4", features = ["serde"] }

# OAuth
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
urlencoding = "2"

# Errors
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Env
dotenvy = "0.15"

# Testing
tokio-test = "0.4"
```

## Инфраструктура
- PostgreSQL 15+ (или 14+)
- Redis 7+ (для rate limiting и сессий)
- Nginx/Traefik (reverse proxy + SSL termination)

---

# Часть 4: Структуры данных

## 4.1 Request/Response структуры

### Регистрация
**Запрос: POST /api/auth/register**
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,              // VARCHAR(255), уникальный
    
    #[validate(length(min = 12, message = "Password must be at least 12 characters"))]
    pub password: String,           // min 12 chars, 1 uppercase, 1 digit, 1 special
    
    pub name: Option<String>,       // VARCHAR(100), опционально
}
```

**Ответ: 201 Created**
```rust
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,              // UUID v4
    pub email: String,
    pub created_at: DateTime<Utc>,  // ISO 8601 timestamp
    pub requires_email_verification: bool,
}
```

### Вход
**Запрос: POST /api/auth/login**
```rust
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,              // VARCHAR(255)
    pub password: String,           // plain text (передаётся по HTTPS)
}
```

**Ответ: 200 OK + Cookies**
```rust
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserDto,              // Public user info (без sensitive данных)
    pub access_token_expires_in: i64, // секунды (обычно 900 = 15 мин)
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub roles: Vec<String>,         // ["user"] или ["user", "admin"]
}
```

### Refresh токен
**Запрос: POST /api/auth/refresh**
```http
Cookie: refresh_token=opaque_random_string_64_chars
```

**Ответ: 200 OK + Новые Cookies**
```rust
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token_expires_in: i64,
}
```

### Текущий пользователь
**Запрос: GET /api/me**
```http
Authorization: Bearer eyJhbGciOiJSUzI1NiIs...
# ИЛИ
Cookie: access_token=eyJhbGciOiJSUzI1NiIs...
```

**Ответ: 200 OK**
```rust
#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,   // ["notes:read", "notes:write"]
}
```

## 4.2 JWT Claims структура

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,                // Subject: user_id (UUID)
    pub email: String,              // Email пользователя
    pub roles: Vec<String>,         // Роли для быстрого доступа
    pub exp: i64,                   // Expiration: UNIX timestamp (15 мин)
    pub iat: i64,                   // Issued at: UNIX timestamp
    pub nbf: i64,                   // Not before: UNIX timestamp
    pub jti: String,                // JWT ID: уникальный id токена (для отзыва)
    pub iss: String,                // Issuer: "your-api.com"
    pub aud: String,                // Audience: "your-app"
    pub typ: String,                // Type: "access"
}
```

**Поля подробно:**
- **sub**: UUID пользователя. Неизменяемый идентификатор.
- **email**: Для отображения, может измениться.
- **roles**: Кэш ролей в токене. Ускоряет проверку без запроса в БД.
- **exp**: Когда токен истечёт. После этого токен отклоняется.
- **iat**: Когда выдан. Полезно для отзыва всех токенов до определённой даты.
- **nbf**: Не валиден до этого времени. Защита от будущих токенов.
- **jti**: Уникальный ID токена. Можно добавить в "чёрный список" для отзыва.
- **iss**: Кто выдал. Защита от токенов других сервисов.
- **aud**: Для кого. Защита от использования токена на другом сервисе.
- **typ**: Различение access/refresh токенов.

## 4.3 Внутренние структуры

### User (БД модель)
```rust
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,                   // PRIMARY KEY
    pub email: String,              // UNIQUE NOT NULL
    pub password_hash: SecretString, // Argon2id hash
    pub name: Option<String>,
    pub email_verified: bool,
    pub roles: Vec<String>,         // JSONB в Postgres
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}
```

### RefreshToken (БД модель)
```rust
#[derive(Debug, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,                   // PRIMARY KEY
    pub user_id: Uuid,              // FOREIGN KEY → users(id)
    pub token_hash: String,         // SHA-256 hash токена (64 hex chars)
    pub expires_at: DateTime<Utc>,  // Когда истечёт
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>, // Когда последний раз использован (ротация)
    pub ip_address: Option<String>, // С какого IP создан
    pub user_agent: Option<String>, // Из какого браузера
}
```

---

# Часть 5: Структура базы данных

## 5.1 Полная SQL схема

```sql
-- Расширения
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Таблица пользователей
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100),
    email_verified BOOLEAN DEFAULT FALSE,
    roles JSONB DEFAULT '["user"]'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    
    CONSTRAINT valid_email CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Индексы для users
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Таблица refresh токенов
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL, -- SHA-256 = 64 hex characters
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    used_at TIMESTAMPTZ,
    ip_address INET,
    user_agent TEXT,
    
    CONSTRAINT valid_token_hash CHECK (LENGTH(token_hash) = 64)
);

-- Критические индексы для refresh_tokens
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at) 
    WHERE expires_at < NOW(); -- Для cleanup job

-- Таблица email verification токенов
CREATE TABLE email_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    used_at TIMESTAMPTZ
);

CREATE INDEX idx_email_verifications_hash ON email_verifications(token_hash);

-- Таблица password reset токенов
CREATE TABLE password_resets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    used_at TIMESTAMPTZ
);

CREATE INDEX idx_password_resets_hash ON password_resets(token_hash);

-- Audit log (для security monitoring)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL, -- 'login', 'logout', 'refresh', 'password_change', etc.
    ip_address INET,
    user_agent TEXT,
    details JSONB, -- дополнительная информация
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- Функция для автоматического обновления updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
```

## 5.2 ER Диаграмма

```
┌─────────────┐       ┌─────────────────┐       ┌─────────────────┐
│    users    │       │  refresh_tokens │       │ email_verifications│
├─────────────┤       ├─────────────────┤       ├─────────────────┤
│ id (PK)     │◄──────┤ user_id (FK)    │       │ user_id (FK)    │
│ email       │       │ token_hash      │       │ token_hash      │
│ password_hash│      │ expires_at      │       │ expires_at      │
│ roles       │       │ created_at      │       └─────────────────┘
│ created_at  │       └─────────────────┘
└─────────────┘                │
                               │
                      ┌────────┴────────┐
                      │   audit_logs    │
                      ├─────────────────┤
                      │ id (PK)         │
                      │ user_id (FK)    │
                      │ action          │
                      │ created_at      │
                      └─────────────────┘
```

---

# Часть 6: Cookies и HTTP Headers

## 6.1 Cookie атрибуты (критически важно!)

### Access Token Cookie
```
Name: access_token
Value: eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
Domain: api.yourapp.com      # Или .yourapp.com для поддоменов
Path: /                      # Доступно для всех API endpoints
HttpOnly: true              # JavaScript НЕ может прочитать (XSS защита)
Secure: true                 # Только HTTPS (man-in-the-middle защита)
SameSite: Strict            # CSRF защита: только с того же сайта
Max-Age: 900               # 15 минут в секундах
Expires: Wed, 15 Apr 2026 12:00:00 GMT
```

### Refresh Token Cookie
```
Name: refresh_token
Value: a1b2c3d4e5f6... (64 character opaque string)
Domain: api.yourapp.com
Path: /api/auth/refresh      # Только для refresh endpoint!
HttpOnly: true
Secure: true
SameSite: Strict
Max-Age: 604800            # 7 дней
Expires: Wed, 22 Apr 2026 12:00:00 GMT
```

## 6.2 HTTP Headers для безопасности

### Response Headers (добавляются всегда)
```http
# Запрещает MIME type sniffing
X-Content-Type-Options: nosniff

# Запрещает отображение в iframe (clickjacking защита)
X-Frame-Options: DENY

# XSS фильтр для старых браузеров
X-XSS-Protection: 1; mode=block

# HSTS: форсирует HTTPS
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload

# CSP: ограничивает источники ресурсов
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'

# Referrer policy
Referrer-Policy: strict-origin-when-cross-origin

# Permissions policy
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

### Request Headers (от клиента)
```http
# Content type
Content-Type: application/json

# Accept
Accept: application/json

# Authorization (если не cookies)
Authorization: Bearer eyJhbGciOiJSUzI1NiIs...
```

---

# Часть 7: API Endpoints полная спецификация

## 7.1 Auth Endpoints

### POST /api/auth/register
**Назначение**: Создание нового аккаунта

**Request:**
```http
POST /api/auth/register HTTP/1.1
Host: api.yourapp.com
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "name": "John Doe"
}
```

**Success 201:**
```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "created_at": "2026-04-14T10:30:00Z",
  "requires_email_verification": true
}
```

**Error 400 (валидация):**
```json
{
  "error": "Validation failed",
  "details": [
    {
      "field": "password",
      "message": "Password must be at least 12 characters"
    }
  ]
}
```

**Error 409 (конфликт):**
```json
{
  "error": "Email already registered"
}
```

### POST /api/auth/login
**Назначение**: Аутентификация и получение токенов

**Request:**
```http
POST /api/auth/login HTTP/1.1
Host: api.yourapp.com
Content-Type: application/json
X-Forwarded-For: 192.168.1.1

{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Success 200:**
```http
HTTP/1.1 200 OK
Content-Type: application/json
Set-Cookie: access_token=eyJhbG...; HttpOnly; Secure; SameSite=Strict; Max-Age=900; Path=/
Set-Cookie: refresh_token=abc123...; HttpOnly; Secure; SameSite=Strict; Max-Age=604800; Path=/api/auth/refresh

{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "roles": ["user"]
  },
  "access_token_expires_in": 900
}
```

**Error 401 (неверные credentials):**
```json
{
  "error": "Invalid email or password"
}
```

**Error 429 (rate limit):**
```json
{
  "error": "Too many attempts. Please try again in 15 minutes."
}
```

### POST /api/auth/refresh
**Назначение**: Получение новой пары токенов

**Request:**
```http
POST /api/auth/refresh HTTP/1.1
Host: api.yourapp.com
Cookie: refresh_token=a1b2c3d4e5f6...
```

**Success 200:**
```http
HTTP/1.1 200 OK
Set-Cookie: access_token=eyJhbGciNEW...; HttpOnly; Secure; SameSite=Strict; Max-Age=900; Path=/
Set-Cookie: refresh_token=new_token...; HttpOnly; Secure; SameSite=Strict; Max-Age=604800; Path=/api/auth/refresh

{
  "access_token_expires_in": 900
}
```

**Error 401 (невалидный refresh):**
```json
{
  "error": "Invalid or expired refresh token"
}
```

### POST /api/auth/logout
**Назначение**: Выход и отзыв токенов

**Request:**
```http
POST /api/auth/logout HTTP/1.1
Host: api.yourapp.com
Cookie: access_token=eyJhbG...; refresh_token=abc123...
```

**Success 200:**
```http
HTTP/1.1 200 OK
Set-Cookie: access_token=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/
Set-Cookie: refresh_token=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/api/auth/refresh

{
  "message": "Logged out successfully"
}
```

### GET /api/auth/me
**Назначение**: Получение текущего пользователя

**Request:**
```http
GET /api/auth/me HTTP/1.1
Host: api.yourapp.com
Cookie: access_token=eyJhbG...
Authorization: Bearer eyJhbG...  # Альтернатива cookie
```

**Success 200:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe",
  "email_verified": true,
  "roles": ["user"],
  "permissions": ["notes:read", "notes:write", "notes:delete_own"],
  "created_at": "2026-04-14T10:30:00Z"
}
```

**Error 401 (нет токена):**
```json
{
  "error": "Authentication required"
}
```

## 7.2 Password Management

### POST /api/auth/forgot-password
**Назначение**: Запрос на сброс пароля

**Request:**
```http
POST /api/auth/forgot-password HTTP/1.1
Content-Type: application/json

{
  "email": "user@example.com"
}
```

**Response 200 (всегда одинаковый для безопасности):**
```json
{
  "message": "If the email exists, a reset link has been sent"
}
```

### POST /api/auth/reset-password
**Назначение**: Сброс пароля по токену

**Request:**
```http
POST /api/auth/reset-password HTTP/1.1
Content-Type: application/json

{
  "token": "reset_token_from_email",
  "new_password": "NewSecurePass123!"
}
```

## 7.3 Email Verification

### POST /api/auth/verify-email
**Назначение**: Подтверждение email

**Request:**
```http
POST /api/auth/verify-email HTTP/1.1
Content-Type: application/json

{
  "token": "verification_token_from_email"
}
```

---

# Часть 8: OAuth интеграция (Google, GitHub, etc.)

## 8.1 Общий флоу OAuth 2.0 + OIDC

```
1. Frontend → GET /api/auth/{provider}/authorize
   ← Возвращает URL для редиректа

2. Браузер → Редирект на Google/GitHub OAuth

3. User → Логин на провайдере

4. Провайдер → Редирект на ваш callback с code

5. Backend → Обмен code на tokens у провайдера

6. Backend → Получение user info (ID token / userinfo endpoint)

7. Backend → Поиск или создание user в вашей БД

8. Backend → Создание ваших JWT токенов

9. Backend → Редирект на frontend с cookies
```

## 8.2 Google OAuth Setup

### Шаг 1: Google Cloud Console
1. https://console.cloud.google.com/
2. Create Project
3. APIs & Services → Credentials
4. Create Credentials → OAuth client ID
5. Application type: Web application
6. Authorized redirect URIs: `https://api.yourapp.com/api/auth/google/callback`
7. Save Client ID and Client Secret

### Шаг 2: Environment
```env
GOOGLE_CLIENT_ID=xxx.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=xxx
GOOGLE_REDIRECT_URL=https://api.yourapp.com/api/auth/google/callback
```

### Шаг 3: Implementation

**GET /api/auth/google/authorize**
```rust
pub async fn google_authorize() -> Result<String, Error> {
    let state = generate_random_state();  // CSRF защита
    let nonce = generate_random_nonce();  // Replay защита
    let code_verifier = generate_code_verifier();  // PKCE
    let code_challenge = sha256(code_verifier);
    
    // Сохранить state, nonce, code_verifier в Redis (5 мин TTL)
    redis.set_ex(format!("oauth:state:{}", state), 
        json!({ "nonce": nonce, "code_verifier": code_verifier }), 
        300
    ).await?;
    
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope=openid+email+profile&\
        state={}&\
        nonce={}&\
        code_challenge={}&\
        code_challenge_method=S256",
        GOOGLE_CLIENT_ID,
        urlencode(GOOGLE_REDIRECT_URL),
        state,
        nonce,
        code_challenge
    );
    
    Ok(auth_url)
}
```

**GET /api/auth/google/callback**
```rust
pub async fn google_callback(
    Query(params): Query<AuthCallbackParams>,  // code, state
) -> Result<impl IntoResponse, Error> {
    // 1. Проверить state (CSRF защита)
    let session_data: OAuthSession = redis
        .get(format!("oauth:state:{}", params.state))
        .await?
        .ok_or(Error::InvalidState)?;
    
    // 2. Обмен code на tokens
    let token_response = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", params.code.as_str()),
            ("client_id", GOOGLE_CLIENT_ID),
            ("client_secret", GOOGLE_CLIENT_SECRET),
            ("redirect_uri", GOOGLE_REDIRECT_URL),
            ("grant_type", "authorization_code"),
            ("code_verifier", &session_data.code_verifier),
        ])
        .send()
        .await?
        .json::<GoogleTokenResponse>()
        .await?;
    
    // 3. Валидировать ID Token
    let id_token = parse_id_token(
        &token_response.id_token,
        GOOGLE_CLIENT_ID,
        &session_data.nonce,
    ).await?;
    
    // 4. Найти или создать пользователя
    let user = match find_user_by_email(&id_token.email).await? {
        Some(user) => user,
        None => {
            // Создать нового пользователя
            create_user(CreateUserRequest {
                email: id_token.email,
                name: id_token.name,
                password: None,  // OAuth user - нет пароля!
                email_verified: id_token.email_verified,
                provider: "google",
                provider_id: id_token.sub,
            }).await?
        }
    };
    
    // 5. Создать сессию
    let (access_token, refresh_token) = create_session(&user).await?;
    
    // 6. Редирект на frontend с cookies
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE, 
        format!("access_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age=900", access_token).parse()?
    );
    headers.insert(
        SET_COOKIE,
        format!("refresh_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age=604800", refresh_token).parse()?
    );
    headers.insert(LOCATION, "https://yourapp.com/dashboard".parse()?);
    
    Ok((StatusCode::FOUND, headers))
}
```

## 8.3 GitHub OAuth Setup

### Шаг 1: GitHub Settings
1. https://github.com/settings/developers
2. OAuth Apps → New OAuth App
3. Authorization callback URL: `https://api.yourapp.com/api/auth/github/callback`
4. Save Client ID and Client Secret

### Шаг 2: Implementation

**Отличия от Google:**
- GitHub не поддерживает OIDC (нет ID token)
- Нужно делать дополнительный запрос на `/user` endpoint

```rust
// Callback handler
pub async fn github_callback(Query(params): Query<AuthCallbackParams>) -> Result<...> {
    // 1. Обмен code на access_token
    let token_res = reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[...])
        .send()
        .await?;
    
    let access_token = token_res.json::<GithubTokenResponse>().await?.access_token;
    
    // 2. Получить user info (GitHub НЕ возвращает ID token!)
    let user_info = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "YourApp")
        .send()
        .await?
        .json::<GithubUser>()
        .await?;
    
    // 3. Найти или создать пользователя
    let user = find_or_create_user_by_provider(
        "github",
        &user_info.id.to_string(),
        &user_info.email,
        &user_info.name,
    ).await?;
    
    // 4. Создать сессию и редирект
    ...
}
```

## 8.4 Другие провайдеры

### Discord
- URL: `https://discord.com/api/oauth2/authorize`
- Scopes: `identify email`
- User endpoint: `https://discord.com/api/users/@me`

### Microsoft (Azure AD)
- URL: `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize`
- Scopes: `openid email profile User.Read`
- Поддерживает OIDC

### Apple Sign In
- Требует Apple Developer Program ($99/год)
- Обязательная 2FA
- Scopes: `name email`
- Особенность: email может быть скрыт (proxy email)

---

# Часть 9: Пошаговое внедрение

## Неделя 1: Фундамент

### День 1: Настройка проекта
```bash
cargo new myapp --bin
cd myapp
cargo add tokio axum serde serde_json sqlx ...
```

- [ ] Создать структуру проекта (src/{auth,db,models,routes,middleware})
- [ ] Настроить подключение к БД (Postgres)
- [ ] Создать .env.example

### День 2: База данных
- [ ] Написать SQL миграции (users, refresh_tokens)
- [ ] Запустить `sqlx migrate run`
- [ ] Создать модели (User, RefreshToken)

### День 3: Регистрация
- [ ] Создать структуры RegisterRequest/Response
- [ ] Валидация (email format, password strength)
- [ ] Argon2id хеширование пароля
- [ ] INSERT в БД
- [ ] Тест в curl/Postman

### День 4: Логин
- [ ] Структуры LoginRequest/Response
- [ ] Проверка Argon2 хеша
- [ ] Генерация JWT (RS256)
- [ ] Генерация refresh токена
- [ ] Установка cookies

### День 5: Middleware
- [ ] Auth middleware (extract + verify JWT)
- [ ] Error handling (401, 403)
- [ ] Protected endpoint /api/me
- [ ] Тест: без токена → 401, с токеном → 200

## Неделя 2: Сессии и OAuth

### День 6: Refresh
- [ ] /api/auth/refresh endpoint
- [ ] Поиск refresh token в БД
- [ ] Ротация (удалить старый, создать новый)
- [ ] Новые cookies

### День 7: Logout
- [ ] Удаление из БД
- [ ] Очистка cookies
- [ ] Тест: после logout access token не работает

### День 8-9: Google OAuth
- [ ] Настройка в Google Cloud Console
- [ ] /authorize endpoint
- [ ] /callback endpoint
- [ ] ID token валидация
- [ ] Link OAuth user с локальным

### День 10: GitHub OAuth
- [ ] Аналогично Google
- [ ] Отличия в API (нет ID token)

## Неделя 3: Безопасность и Polish

### День 11: Rate Limiting
- [ ] Tower Governor или кастом
- [ ] Лимиты: login 5/15min, api 100/min
- [ ] Хранение счётчиков в Redis

### День 12: Security Headers
- [ ] Helmet middleware
- [ ] CORS policy
- [ ] HSTS

### День 13: Email функции
- [ ] Email verification
- [ ] Password reset
- [ ] Интеграция с SendGrid/AWS SES

### День 14: Audit Logging
- [ ] Таблица audit_logs
- [ ] Middleware для логирования
- [ ] Мониторинг failed login attempts

### День 15: Тестирование
- [ ] Unit tests (хеширование, JWT)
- [ ] Integration tests (end-to-end flow)
- [ ] Security tests (XSS, CSRF попытки)

---

# Часть 10: Частые ошибки и решения

## Ошибка 1: Хранение паролей в plaintext
**Проблема**: `password VARCHAR(255)` без хеширования
**Решение**: Использовать Argon2id
```rust
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();
```

## Ошибка 2: JWT в localStorage
**Проблема**: XSS атака может украсть токен
**Решение**: HttpOnly cookies
```rust
Cookie::build("access_token", token)
    .http_only(true)
    .secure(true)
    .same_site(SameSite::Strict)
    .finish()
```

## Ошибка 3: Нет ротации refresh токенов
**Проблема**: Refresh token украден → вечный доступ
**Решение**: Удалять старый при использовании

## Ошибка 4: Отсутствие rate limiting
**Проблема**: Brute force на /login
**Решение**: 5 попыток на IP за 15 минут

## Ошибка 5: Слабые пароли
**Проблема**: Пользователи используют "password123"
**Решение**: Валидация сложности + haveibeenpwned API

---

# Резюме

## Что мы построили
- Полноценную систему аутентификации с JWT
- Безопасное хранение паролей (Argon2id)
- Управление сессиями с ротацией
- OAuth интеграцию (Google, GitHub)
- Защиту от основных атак (CSRF, XSS, brute force)

## Архитектура
- Stateless: JWT для масштабируемости
- Stateful sessions: refresh токены в БД для контроля
- Layered security: middleware stack

## Ключевые файлы
- `src/auth/mod.rs` - логика аутентификации
- `src/middleware/auth.rs` - проверка JWT
- `src/models/user.rs` - структуры данных
- `migrations/*.sql` - схема БД

## Следующие шаги
1. Добавить 2FA (TOTP)
2. WebAuthn (FaceID/TouchID)
3. SSO для enterprise (SAML)
4. Аналитика (retention, failed login patterns)

---

**Запомни**: Безопасность - это процесс, а не пункт в чек-листе. Регулярно аудируй код и следи за новыми уязвимостями.
