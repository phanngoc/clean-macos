# Example: User Authentication with 2FA

This is a complete example showing what spec-builder generates for a typical feature request.

## Original Request

> "Implement user authentication with 2FA, email verification, and session expiry handling in our web app."

---

## Generated Output Preview

### 📄 requirements.md (Sample)

```markdown
# Requirements: User Authentication with 2FA

## Overview
Implement a secure authentication system that supports email/password login, 
two-factor authentication (2FA), email verification, and automatic session expiry.

## User Stories

### Story 1: User Registration
**As a** new user
**I want** to register with my email and password
**So that** I can create an account and access the application

### Story 2: Email Verification
**As a** registered user
**I want** to verify my email address
**So that** I can prove ownership and activate my account

### Story 3: Two-Factor Authentication
**As a** security-conscious user
**I want** to enable 2FA on my account
**So that** my account is protected even if my password is compromised

## Acceptance Criteria

### AC-1: User Registration
**WHEN** a user submits registration form
**THE SYSTEM SHALL** create an account and send verification email

**GIVEN** valid email and password
**WHEN** user clicks "Register"
**THEN** account is created in "unverified" state
**AND** verification email is sent within 30 seconds
**AND** user receives confirmation message

### AC-2: Email Verification
**WHEN** user clicks verification link in email
**THE SYSTEM SHALL** activate the account

**GIVEN** unverified account
**WHEN** user clicks valid verification link
**THEN** account status changes to "verified"
**AND** user can log in
**AND** verification link expires after use

### AC-3: 2FA Enrollment
**WHEN** verified user enables 2FA
**THE SYSTEM SHALL** generate TOTP secret and QR code

**GIVEN** verified and logged-in user
**WHEN** user navigates to security settings and enables 2FA
**THEN** system generates TOTP secret
**AND** displays QR code for authenticator app
**AND** provides 10 backup codes
**AND** requires verification before activation

## Constraints & Requirements

### Functional Requirements
- REQ-001: Support email/password authentication
- REQ-002: Send verification emails within 30 seconds
- REQ-003: Support TOTP-based 2FA (RFC 6238)
- REQ-004: Generate 10 single-use backup codes
- REQ-005: Sessions expire after 24 hours of inactivity
- REQ-006: Support "remember me" for 30-day sessions

### Non-Functional Requirements
- Performance: Login response < 500ms (95th percentile)
- Security: Passwords hashed with bcrypt (cost factor 12)
- Security: TOTP secrets encrypted at rest
- Availability: 99.9% uptime for authentication service
- Scalability: Support 1000 concurrent authentications

### Input Requirements
- Email: Valid RFC 5322 format, max 255 characters
- Password: Min 8 characters, must include uppercase, lowercase, number
- 2FA Code: 6 digits, valid for 30 seconds

### Output Requirements
- Success: Return JWT token with user claims
- Failure: Return specific error codes (401, 403, 429)
- Session token: Include user ID, roles, expiry timestamp

## Edge Cases
1. User tries to register with existing email
2. Verification link clicked multiple times
3. Verification link expired (>24 hours)
4. 2FA code entered after 30-second window
5. All backup codes used, 2FA device lost
6. Session expires during active request
7. Multiple login attempts with wrong password
8. User forgets password, 2FA enabled

## Out of Scope
- Social login (Google, Facebook, etc.)
- Biometric authentication
- SMS-based 2FA
- Password reset via security questions
- Multi-device session management
```

---

### 🏗️ design.md (Sample)

```markdown
# Technical Design: User Authentication with 2FA

## Overview
Implement a stateless JWT-based authentication system with TOTP 2FA support,
email verification workflow, and Redis-backed session management.

## Architecture

### System Architecture
```
┌──────────────────────────────────────────────────────────┐
│                        Frontend                          │
│  (React - Auth Context, Protected Routes)                │
└────────────────┬─────────────────────────────────────────┘
                 │ HTTPS/REST
┌────────────────▼─────────────────────────────────────────┐
│                     API Gateway                          │
│          (Rate Limiting, JWT Validation)                 │
└────────┬───────────────────────────────────┬─────────────┘
         │                                   │
┌────────▼──────────────┐         ┌─────────▼──────────────┐
│  Auth Service         │         │   Email Service        │
│  - Login              │         │   - Send verification  │
│  - Register           │◄────────┤   - Send alerts        │
│  - 2FA Operations     │         │                        │
└────────┬──────────────┘         └────────────────────────┘
         │
┌────────▼──────────────┬─────────────────────────────────┐
│     Database          │     Redis Cache                 │
│  - User accounts      │  - Sessions                     │
│  - 2FA secrets        │  - Rate limiting                │
│  - Backup codes       │  - Email tokens                 │
└───────────────────────┴─────────────────────────────────┘
```

### Component Breakdown

#### Component A: AuthService
- **Purpose**: Core authentication logic
- **Responsibilities**: 
  - Validate credentials
  - Generate and verify JWT tokens
  - Manage 2FA enrollment and verification
  - Handle session lifecycle
- **Dependencies**: UserRepository, TokenService, CryptoService
- **Interfaces**: 
  - `register(email, password): User`
  - `login(email, password): SessionToken`
  - `verify2FA(userId, code): boolean`
  - `enrollIn2FA(userId): { qrCode, backupCodes }`

#### Component B: TokenService
- **Purpose**: JWT token management
- **Responsibilities**: 
  - Generate JWT access tokens
  - Generate refresh tokens
  - Validate and decode tokens
  - Handle token expiry
- **Dependencies**: CryptoService, ConfigService
- **Interfaces**: 
  - `generateAccessToken(userId, claims): string`
  - `validateToken(token): TokenPayload`
  - `refreshToken(refreshToken): string`

#### Component C: EmailVerificationService
- **Purpose**: Handle email verification workflow
- **Responsibilities**: 
  - Generate verification tokens
  - Send verification emails
  - Validate verification tokens
  - Update user verification status
- **Dependencies**: EmailService, TokenStore (Redis)
- **Interfaces**: 
  - `sendVerificationEmail(userId, email): void`
  - `verifyEmail(token): boolean`

#### Component D: TwoFactorService
- **Purpose**: Manage TOTP-based 2FA
- **Responsibilities**: 
  - Generate TOTP secrets
  - Create QR codes
  - Verify TOTP codes
  - Manage backup codes
- **Dependencies**: CryptoService, UserRepository
- **Interfaces**: 
  - `generateSecret(): string`
  - `generateQRCode(secret, email): string`
  - `verifyTOTP(secret, code): boolean`
  - `generateBackupCodes(): string[]`

## Data Model

### Entities

```typescript
Entity: User
Fields:
  - id: UUID (primary key)
  - email: string (unique, indexed)
  - passwordHash: string (bcrypt)
  - emailVerified: boolean
  - twoFactorEnabled: boolean
  - twoFactorSecret: string (encrypted, nullable)
  - createdAt: timestamp
  - updatedAt: timestamp
Relationships:
  - hasMany: Sessions
  - hasMany: BackupCodes

Entity: Session
Fields:
  - id: UUID (primary key)
  - userId: UUID (foreign key)
  - token: string (JWT)
  - refreshToken: string
  - expiresAt: timestamp
  - lastActivityAt: timestamp
  - ipAddress: string
  - userAgent: string
Relationships:
  - belongsTo: User

Entity: BackupCode
Fields:
  - id: UUID (primary key)
  - userId: UUID (foreign key)
  - codeHash: string (bcrypt)
  - used: boolean
  - usedAt: timestamp (nullable)
Relationships:
  - belongsTo: User

Entity: EmailVerificationToken (Redis)
Fields:
  - token: string (key)
  - userId: UUID
  - email: string
  - expiresAt: timestamp (24 hours TTL)
```

## Sequence Diagrams

### Primary Flow: Registration with Email Verification
```
User → Frontend: Fill registration form
Frontend → API: POST /api/auth/register {email, password}
API → AuthService: register(email, password)
AuthService → Database: Create user (emailVerified=false)
Database → AuthService: User created
AuthService → EmailService: sendVerificationEmail(userId, email)
EmailService → Redis: Store verification token (24h TTL)
EmailService → SMTP: Send email with link
SMTP → User: Verification email delivered
AuthService → API: {userId, message: "Check email"}
API → Frontend: Registration success
Frontend → User: "Please verify your email"

[Later...]
User → Email: Click verification link
Browser → API: GET /api/auth/verify-email?token=xxx
API → EmailService: verifyEmail(token)
EmailService → Redis: Get userId by token
Redis → EmailService: userId found
EmailService → Database: Update user.emailVerified = true
EmailService → Redis: Delete token
EmailService → API: Verification success
API → Browser: Redirect to login page
```

### Primary Flow: Login with 2FA
```
User → Frontend: Enter email + password
Frontend → API: POST /api/auth/login {email, password}
API → AuthService: login(email, password)
AuthService → Database: Find user by email
Database → AuthService: User found
AuthService → CryptoService: verifyPassword(password, hash)
CryptoService → AuthService: Password correct
AuthService: Check if 2FA enabled
AuthService → API: {requiresTwoFactor: true, tempToken: xxx}
API → Frontend: Require 2FA

User → Authenticator App: Get 6-digit code
User → Frontend: Enter 2FA code
Frontend → API: POST /api/auth/verify-2fa {tempToken, code}
API → AuthService: verify2FA(userId, code)
AuthService → Database: Get user 2FA secret
Database → AuthService: Secret retrieved
AuthService → TwoFactorService: verifyTOTP(secret, code)
TwoFactorService: Verify code (30-second window)
TwoFactorService → AuthService: Code valid
AuthService → TokenService: generateAccessToken(userId)
TokenService → AuthService: JWT token
AuthService → Redis: Store session
AuthService → API: {accessToken, refreshToken}
API → Frontend: Login success
Frontend → User: Redirect to dashboard
```

## API Contracts

### POST /api/auth/register
- **Request Body**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123"
}
```
- **Response (201)**:
```json
{
  "status": "success",
  "data": {
    "userId": "uuid",
    "email": "user@example.com",
    "message": "Please check your email to verify your account"
  }
}
```
- **Error Codes**: 
  - 400: Invalid email or weak password
  - 409: Email already registered
  - 429: Too many registration attempts

### POST /api/auth/login
- **Request Body**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123"
}
```
- **Response (200)** - If 2FA disabled:
```json
{
  "status": "success",
  "data": {
    "accessToken": "jwt-token",
    "refreshToken": "refresh-token",
    "expiresIn": 86400
  }
}
```
- **Response (200)** - If 2FA enabled:
```json
{
  "status": "2fa_required",
  "data": {
    "tempToken": "temp-token",
    "message": "Please enter your 2FA code"
  }
}
```

### POST /api/auth/2fa/enroll
- **Headers**: Authorization: Bearer {accessToken}
- **Response (200)**:
```json
{
  "status": "success",
  "data": {
    "qrCode": "data:image/png;base64,...",
    "secret": "BASE32SECRET",
    "backupCodes": [
      "12345678",
      "87654321",
      ...
    ]
  }
}
```

## Technical Decisions

### Decision 1: JWT vs Session Cookies
- **Context**: Need to choose session management approach
- **Options Considered**:
  1. JWT tokens: Stateless, scalable, works with mobile apps
  2. Session cookies: Stateful, easier to revoke, traditional
- **Decision**: JWT with refresh tokens stored in Redis
- **Rationale**: 
  - Stateless tokens scale horizontally
  - Redis provides fast revocation when needed
  - Works seamlessly with mobile apps and SPA
- **Trade-offs**: 
  - Slightly more complex revocation
  - Tokens can't be invalidated until expiry (mitigated with short TTL + refresh)

### Decision 2: TOTP vs SMS for 2FA
- **Context**: Choose 2FA implementation method
- **Options Considered**:
  1. TOTP (Time-based OTP): App-based, offline, free
  2. SMS: User-friendly, but costs money and security concerns
- **Decision**: TOTP (RFC 6238)
- **Rationale**:
  - No ongoing costs
  - More secure than SMS
  - Works offline
  - Industry standard (Google Authenticator, Authy)
- **Trade-offs**: 
  - Less convenient for non-technical users
  - Requires smartphone with authenticator app

### Decision 3: Password Hashing Algorithm
- **Decision**: bcrypt with cost factor 12
- **Rationale**:
  - Industry standard for password hashing
  - Resistant to brute force (adaptive cost)
  - Cost factor 12 balances security and performance (~300ms per hash)

## Security Considerations
- **Authentication**: JWT tokens with 1-hour expiry, refresh tokens with 30-day expiry
- **Authorization**: Role-based access control (RBAC) in JWT claims
- **Data Protection**: 
  - Passwords: bcrypt (cost 12)
  - 2FA secrets: AES-256-GCM encryption at rest
  - Tokens: SHA-256 hashing for refresh tokens
- **Input Validation**: 
  - Email: RFC 5322 validation + DNS MX record check
  - Password: Entropy check, common password blacklist
  - Rate limiting: Max 5 failed login attempts per 15 minutes per IP

## Performance Considerations
- **Expected Load**: 1000 auth requests/minute peak
- **Optimization Strategy**: 
  - Redis cache for sessions and rate limiting
  - Database read replicas for user lookups
  - JWT validation is CPU-bound, use efficient library
- **Caching Strategy**: 
  - User sessions: Redis with TTL
  - Failed login attempts: Redis with 15-min TTL
  - Email verification tokens: Redis with 24-hour TTL

## Risks & Mitigations

### Risk 1: Email Delivery Failures
- **Impact**: High (users can't verify accounts)
- **Probability**: Medium
- **Mitigation**: 
  - Use reliable email service (SendGrid/AWS SES)
  - Implement retry mechanism
  - Provide "Resend verification email" option
  - Log all email delivery failures
  - Monitor delivery rates

### Risk 2: 2FA Device Loss
- **Impact**: High (users locked out of account)
- **Probability**: Medium
- **Mitigation**: 
  - Provide backup codes during enrollment
  - Implement account recovery workflow
  - Support multiple 2FA devices per account (future)
  - Clear documentation on backup code storage

### Risk 3: Token Leakage
- **Impact**: Critical (account compromise)
- **Probability**: Low
- **Mitigation**: 
  - Short-lived access tokens (1 hour)
  - Refresh token rotation
  - Token revocation endpoint
  - Monitor for suspicious activity
  - Implement device fingerprinting
```

---

### ✅ tasks.md (Sample - Abbreviated)

```markdown
# Task Breakdown: User Authentication with 2FA

## Task Overview
Total estimated tasks: 24
Estimated total effort: 80-120 hours

## Task Dependencies Graph
```
Task 1 → Task 3, Task 4
Task 2 → Task 5, Task 6
Task 3 → Task 7
Task 4 → Task 8
Task 7, Task 8 → Task 11
...
```

---

## 📦 Phase 1: Foundation & Data Model

### Task #1: Design and Create Database Schema

**Category**: Database

**Description**:
Create database migrations for User, Session, and BackupCode tables with proper
indexes, constraints, and relationships.

**Expected Output**:
- Migration files for all tables
- Schema documentation
- Index strategy document

**Input Requirements**:
- Database type (PostgreSQL assumed)
- ORM framework (e.g., Prisma, TypeORM)

**Dependencies**:
- None

**Acceptance Criteria**:
- [ ] Users table with email (unique), passwordHash, emailVerified, 2FA fields
- [ ] Sessions table with userId foreign key, token fields, timestamps
- [ ] BackupCodes table with userId foreign key, codeHash, used flag
- [ ] Proper indexes on email, userId, token fields
- [ ] Foreign key constraints with CASCADE delete
- [ ] Migration rollback scripts tested

**Complexity**: Medium
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Use UUID for primary keys
- Add created_at/updated_at timestamps to all tables
- Consider partitioning Sessions table by date for performance
- Encrypt 2FA secrets at database level if possible

---

### Task #2: Set Up Redis for Session Management

**Category**: Infrastructure

**Description**:
Configure Redis instance for session storage, rate limiting, and temporary tokens.
Set up connection pooling and error handling.

**Expected Output**:
- Redis connection configuration
- Redis client wrapper with error handling
- Health check endpoint for Redis

**Input Requirements**:
- Redis server credentials
- Environment configuration

**Dependencies**:
- None

**Acceptance Criteria**:
- [ ] Redis client configured with connection pooling
- [ ] TTL-based expiry working correctly
- [ ] Graceful handling of Redis connection failures
- [ ] Health check endpoint returns Redis status
- [ ] Documentation for Redis key naming conventions

**Complexity**: Low
**Estimated Time**: 2-3 hours

---

## 📦 Phase 2: Core Authentication Logic

### Task #3: Implement Password Hashing Service

**Category**: Backend/Security

**Description**:
Create a CryptoService module that handles bcrypt password hashing and verification
with configurable cost factor.

**Expected Output**:
- CryptoService class/module
- Unit tests with various password scenarios
- Performance benchmarks

**Input Requirements**:
- bcrypt library installed

**Dependencies**:
- Task #1 (database schema)

**Acceptance Criteria**:
- [ ] hashPassword(password): Promise<string>
- [ ] verifyPassword(password, hash): Promise<boolean>
- [ ] Cost factor configurable via environment (default 12)
- [ ] Async operations to avoid blocking
- [ ] Unit tests cover: valid passwords, invalid passwords, edge cases
- [ ] Performance: <500ms per hash on standard hardware

**Complexity**: Low
**Estimated Time**: 3-4 hours

**Implementation Notes**:
- Use bcrypt.compare() not manual comparison
- Add timing-attack protection
- Log hash generation time for monitoring

---

### Task #4: Implement JWT Token Service

**Category**: Backend/Security

**Description**:
Create TokenService for generating and validating JWT access tokens and refresh tokens.

**Expected Output**:
- TokenService class/module
- Token generation and validation functions
- Unit tests

**Input Requirements**:
- JWT library (e.g., jsonwebtoken)
- Secret keys from environment

**Dependencies**:
- Task #1 (database schema for user data)

**Acceptance Criteria**:
- [ ] generateAccessToken(userId, claims): string (1-hour expiry)
- [ ] generateRefreshToken(userId): string (30-day expiry)
- [ ] validateToken(token): TokenPayload | null
- [ ] Tokens include: userId, email, roles, iat, exp
- [ ] RS256 signing algorithm
- [ ] Proper error handling for expired/invalid tokens
- [ ] Unit tests cover: valid tokens, expired tokens, tampered tokens

**Complexity**: Medium
**Estimated Time**: 4-5 hours

---

## 📦 Phase 3: Registration & Email Verification

### Task #7: Implement User Registration Endpoint

**Category**: Backend/API

**Description**:
Create POST /api/auth/register endpoint that validates input, creates user,
and triggers email verification.

**Expected Output**:
- Registration API endpoint
- Input validation middleware
- Integration tests

**Input Requirements**:
- Database and Redis configured
- Email service ready

**Dependencies**:
- Task #3 (password hashing)
- Task #1 (database schema)

**Acceptance Criteria**:
- [ ] POST /api/auth/register accepts email + password
- [ ] Email validation (RFC 5322 format + unique check)
- [ ] Password validation (min 8 chars, complexity rules)
- [ ] User created with emailVerified=false
- [ ] Email verification triggered automatically
- [ ] Returns 201 with userId and message
- [ ] Error handling: 400 (invalid input), 409 (duplicate email)
- [ ] Rate limiting: max 5 registrations per IP per hour
- [ ] Integration tests cover happy path and error cases

**Complexity**: High
**Estimated Time**: 6-8 hours

---

[... Continue with remaining 17 tasks ...]

---

## Summary

### Critical Path
1. Task #1 (Database Schema)
2. Task #3 (Password Hashing)
3. Task #7 (Registration API)
4. Task #11 (Login API)
5. Task #15 (2FA Enrollment)
6. Task #18 (2FA Verification)

### Parallel Work Opportunities
- Task #2 (Redis) can be done parallel to Task #1
- Task #4 (JWT) can be done parallel to Task #3
- Frontend tasks (Phase 5) can start once backend contracts are finalized
- Testing tasks can be written before implementation

### Risk Items
- Task #8 (Email Service): External dependency, delivery failures possible
- Task #15 (2FA Enrollment): Complex crypto operations, security critical
- Task #20 (Session Expiry): Background job scheduling, needs monitoring

### Definition of Done
- [ ] All 24 tasks completed
- [ ] Unit tests pass (>80% coverage)
- [ ] Integration tests pass
- [ ] API documentation complete
- [ ] Security audit passed
- [ ] Performance tests meet requirements (<500ms login)
- [ ] Deployed to staging environment
- [ ] User acceptance testing completed
```

---

## How This Example Was Generated

This complete specification was generated by the spec-builder skill from a single sentence:

> "Implement user authentication with 2FA, email verification, and session expiry handling in our web app."

The skill automatically:
1. ✅ Identified user stories and acceptance criteria
2. ✅ Designed a complete technical architecture
3. ✅ Created detailed API contracts and data models
4. ✅ Generated 24 granular, actionable tasks
5. ✅ Documented trade-offs and risks
6. ✅ Provided implementation guidance

All three documents (requirements.md, design.md, tasks.md) are now ready for:
- Product team review
- Developer implementation
- AI coding agent consumption
- Stakeholder approval

