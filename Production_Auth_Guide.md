# Руководство по Production Authentication

## 🔐 Архитектура

```
Клиент → API Gateway → Auth Middleware → Handler → БД
```

## 📊 Эндпоинты и данные

### POST /api/auth/register
**Запрос:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Валидация:**
- email: формат email, уникальный
- password: мин 12 символов, 1 заглавная, 1 цифра, 1 спецсимвол

**Ответ 201:**
```json
{
  "user_id": "uuid",
  "message": "Check email for verification"
}
```

**Ошибки:**
- 400: Невалидные данные
- 409: Email уже существует
- 429: Слишком много попыток

---

### POST /api/auth/login
**Запрос:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Ответ 200 + Cookies:**
```http
Set-Cookie: access_token=eyJhbG...; HttpOnly; Secure; SameSite=Strict; Max-Age=900
Set-Cookie: refresh_token=abc123...; HttpOnly; Secure; SameSite=Strict; Max-Age=604800
```

**Тело ответа:**
```json
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "roles": ["user"]
  }
}
```

**Ошибки:**
- 401: Неверные credentials
- 429: Rate limit

---

### POST /api/auth/refresh
**Cookies:**
```
Cookie: refresh_token=abc123...
```

**Логика сервера:**
1. Найти refresh_token_hash в БД
2. Проверить expires_at > now()
3. Удалить старый токен (ротация!)
4. Сгенерировать новую пару
5. Вернуть в cookies

**Ответ 200 + Новые Cookies**

**Ошибки:**
- 401: Токен не найден или истёк

---

### POST /api/auth/logout
**Cookies:**
```
Cookie: access_token=...; refresh_token=...
```

**Логика:**
1. Найти refresh_token в БД
2. Удалить из БД (отмена сессии)
3. Очистить cookies (Max-Age=0)

**Ответ 200:**
```json
{
  "message": "Logged out successfully"
}
```

---

### GET /api/me (Protected)
**Headers:**
```
Cookie: access_token=eyJhbG...
```

**Middleware проверяет:**
1. JWT подпись (RS256)
2. exp > now()
3. iss в whitelist
4. aud совпадает

**Ответ 200:**
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "name": "User Name",
  "roles": ["user"],
  "permissions": ["notes:read", "notes:write"]
}
```

**Ошибки:**
- 401: Токен отсутствует/невалидный/истёк

---

## 🗄️ Структура БД

### Таблица users
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

**Типы полей:**
- id: UUID (16 bytes)
- email: VARCHAR(255), индекс UNIQUE
- password_hash: VARCHAR(255) - Argon2id hash
- email_verified: BOOLEAN
- created_at/updated_at: TIMESTAMP WITH TIME ZONE

### Таблица refresh_tokens
```sql
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL, -- SHA-256 hash
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    used_at TIMESTAMP  -- для отслеживания ротации
);

CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
```

**Типы полей:**
- token_hash: SHA-256 (64 hex chars)
- expires_at: TIMESTAMP (refresh token lifetime: 7-30 дней)

---

## 🔑 Токены

### Access Token (JWT)
**Header:**
```json
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "key-id-123"
}
```

**Payload:**
```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "roles": ["user"],
  "exp": 1744681200,
  "iat": 1744680300,
  "nbf": 1744680300,
  "jti": "unique-token-id",
  "iss": "https://your-api.com",
  "aud": "your-app"
}
```

**Типы полей:**
- sub: UUID (user id)
- exp: UNIX timestamp (15 минут)
- iat: UNIX timestamp (когда выдан)
- jti: UUID (unique token id для отзыва)

### Refresh Token
**Формат:** Криптографически случайная строка
**Размер:** 32 байта (64 hex chars)
**Хранение:** Только SHA-256 hash в БД
**Время жизни:** 7-30 дней
**Ротация:** Новый токен при каждом использовании

---

## 🍪 Cookies

### Access Token Cookie
```
Name: access_token
Value: JWT string
HttpOnly: true
Secure: true (только HTTPS)
SameSite: Strict
Max-Age: 900 (15 минут)
Path: /
```

### Refresh Token Cookie
```
Name: refresh_token
Value: Opaque string (32 bytes)
HttpOnly: true
Secure: true
SameSite: Strict
Max-Age: 604800 (7 дней)
Path: /api/auth/refresh
```

---

## 🛡️ Security

### Password Hashing (Argon2id)
```rust
Argon2::new(
    Algorithm::Argon2id,
    Version::V0x13,
    Params::new(65536, 3, 4, None).unwrap()
)
```

**Параметры:**
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4

### JWT Signing (RS256)
- Private key: На сервере (PEM файл)
- Public key: Для верификации
- Key rotation: Каждые 90 дней

### Rate Limits
| Endpoint | Limit | Window |
|----------|-------|--------|
| /login | 5 | 15 min |
| /register | 3 | 1 hour |
| /refresh | 10 | 5 min |
| API общий | 100 | 1 min |

---

## 📦 Middleware Stack (порядок)

```rust
1. CORS (origins whitelist)
2. SecurityHeaders
3. RequestID
4. RateLimit
5. Logger
6. Auth (verify JWT)
7. RBAC (check permissions)
8. Handler
```

---

## 🔄 Полный флоу авторизации

### Первый вход
1. POST /login → Credentials
2. Сервер: Проверить Argon2 hash
3. Сервер: Создать session в БД
4. Сервер: Сгенерировать JWT + refresh token
5. Ответ: Cookies с токенами

### Использование API
1. Браузер: Запрос с access_token cookie
2. Middleware: Проверить JWT подпись + exp
3. Handler: Выполнить запрос

### Токен истёк (401)
1. Фронт: POST /refresh с refresh_token cookie
2. Сервер: Найти hash в БД
3. Сервер: Удалить старый, создать новый
4. Ответ: Новые cookies

### Выход
1. POST /logout
2. Сервер: Удалить refresh из БД
3. Ответ: Пустые cookies (Max-Age=0)

---

## ⚠️ Коды ошибок

| Код | Когда | Действие фронта |
|-----|-------|-----------------|
| 200 | Успех | Продолжить |
| 400 | Невалидные данные | Показать ошибки валидации |
| 401 | Нет/истёк токен | Редирект на логин или auto-refresh |
| 403 | Нет прав | Показать "Доступ запрещён" |
| 409 | Конфликт (email exists) | Показать сообщение |
| 429 | Rate limit | Подождать, повторить |
| 500 | Ошибка сервера | Сообщить админу |

---

## 📝 Чек-лист внедрения

- [ ] Таблицы users и refresh_tokens
- [ ] Argon2id для паролей
- [ ] RS256 для JWT
- [ ] HttpOnly Secure SameSite cookies
- [ ] Ротация refresh токенов
- [ ] Rate limiting (per IP + per user)
- [ ] Audit logging (login, logout, refresh)
- [ ] HTTPS only
- [ ] Security headers
- [ ] CORS whitelist

#auth #backend #rust #jwt #security
