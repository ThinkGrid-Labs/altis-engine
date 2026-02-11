# API Structure & Security

## API Endpoint Organization

### Customer-Facing APIs (`/v1/*`)

**Authentication**: Customer JWT tokens  
**Rate Limiting**: 100 requests/minute per IP  
**Purpose**: Public booking and order management

```
/v1/auth/login
/v1/auth/register
/v1/offers/search
/v1/offers/{id}/accept
/v1/orders/{id}
/v1/orders/{id}/pay
/v1/orders/{id}/customize
```

---

### Admin/CMS APIs (`/v1/admin/*`)

**Authentication**: Admin JWT tokens with role-based access  
**Rate Limiting**: 1000 requests/minute per user  
**Purpose**: Airline configuration and management

#### Airline Management
```
GET    /v1/admin/airlines
POST   /v1/admin/airlines
GET    /v1/admin/airlines/{id}
PUT    /v1/admin/airlines/{id}
DELETE /v1/admin/airlines/{id}
```

#### Product Management
```
GET    /v1/admin/airlines/{airline_id}/products
POST   /v1/admin/airlines/{airline_id}/products
GET    /v1/admin/products/{id}
PUT    /v1/admin/products/{id}
DELETE /v1/admin/products/{id}
POST   /v1/admin/airlines/{airline_id}/products/bulk
```

#### Pricing Rules
```
GET    /v1/admin/airlines/{airline_id}/pricing-rules
POST   /v1/admin/airlines/{airline_id}/pricing-rules
GET    /v1/admin/pricing-rules/{id}
PUT    /v1/admin/pricing-rules/{id}
DELETE /v1/admin/pricing-rules/{id}
POST   /v1/admin/pricing-rules/{id}/test
```

#### Bundle Templates
```
GET    /v1/admin/airlines/{airline_id}/bundles
POST   /v1/admin/airlines/{airline_id}/bundles
GET    /v1/admin/bundles/{id}
PUT    /v1/admin/bundles/{id}
DELETE /v1/admin/bundles/{id}
POST   /v1/admin/bundles/{id}/test
```

#### Business Rules
```
GET    /v1/admin/airlines/{airline_id}/rules
POST   /v1/admin/airlines/{airline_id}/rules
GET    /v1/admin/rules/{id}
PUT    /v1/admin/rules/{id}
PATCH  /v1/admin/rules/{id}/status
DELETE /v1/admin/rules/{id}
```

#### Inventory Rules
```
GET    /v1/admin/airlines/{airline_id}/inventory-rules
POST   /v1/admin/airlines/{airline_id}/inventory-rules
GET    /v1/admin/inventory-rules/{id}
PUT    /v1/admin/inventory-rules/{id}
```

#### Flight Management
```
GET    /v1/admin/airlines/{airline_id}/flights
POST   /v1/admin/airlines/{airline_id}/flights
GET    /v1/admin/flights/{id}
PUT    /v1/admin/flights/{id}
DELETE /v1/admin/flights/{id}
POST   /v1/admin/airlines/{airline_id}/flights/bulk
```

#### Analytics & Reports
```
GET    /v1/admin/airlines/{airline_id}/analytics/revenue
GET    /v1/admin/airlines/{airline_id}/analytics/conversion
GET    /v1/admin/airlines/{airline_id}/reports/orders
GET    /v1/admin/airlines/{airline_id}/reports/inventory
```

#### Audit Logs
```
GET    /v1/admin/airlines/{airline_id}/audit-logs
GET    /v1/admin/audit-logs/{id}
```

---

## Authentication & Authorization

### Customer Authentication

**Login**:
```bash
POST /v1/auth/login
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "user": {
    "id": "user-123",
    "email": "user@example.com",
    "role": "CUSTOMER"
  }
}
```

**JWT Claims**:
```json
{
  "sub": "user-123",
  "email": "user@example.com",
  "role": "CUSTOMER",
  "exp": 1717234567
}
```

---

### Admin Authentication

**Login**:
```bash
POST /v1/admin/auth/login
{
  "email": "admin@americanairlines.com",
  "password": "admin123"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 7200,
  "user": {
    "id": "admin-456",
    "email": "admin@americanairlines.com",
    "role": "ADMIN",
    "airline_id": "airline-aa",
    "permissions": [
      "manage_products",
      "manage_pricing",
      "manage_bundles",
      "view_analytics"
    ]
  }
}
```

**JWT Claims**:
```json
{
  "sub": "admin-456",
  "email": "admin@americanairlines.com",
  "role": "ADMIN",
  "airline_id": "airline-aa",
  "permissions": ["manage_products", "manage_pricing", "manage_bundles", "view_analytics"],
  "exp": 1717241767
}
```

---

## Role-Based Access Control (RBAC)

### Roles

| Role | Description | Access |
|------|-------------|--------|
| **CUSTOMER** | End user booking flights | `/v1/offers/*`, `/v1/orders/*` |
| **ADMIN** | Airline administrator | `/v1/admin/*` (scoped to their airline) |
| **SUPER_ADMIN** | Platform administrator | `/v1/admin/*` (all airlines) |
| **ANALYST** | Read-only analytics access | `/v1/admin/*/analytics/*`, `/v1/admin/*/reports/*` |

### Permissions

| Permission | Description | Endpoints |
|------------|-------------|-----------|
| `manage_products` | Create/edit/delete products | `/v1/admin/*/products/*` |
| `manage_pricing` | Configure pricing rules | `/v1/admin/*/pricing-rules/*` |
| `manage_bundles` | Configure bundle templates | `/v1/admin/*/bundles/*` |
| `manage_flights` | Add/edit flight schedules | `/v1/admin/*/flights/*` |
| `view_analytics` | View reports and analytics | `/v1/admin/*/analytics/*` |
| `manage_users` | Manage admin users | `/v1/admin/users/*` |

---

## Middleware Implementation

### Admin Authentication Middleware

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract token from Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 2. Decode and validate JWT
    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(state.auth.secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 3. Check role
    if token_data.claims.role != "ADMIN" && token_data.claims.role != "SUPER_ADMIN" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 4. Inject claims into request extensions
    req.extensions_mut().insert(token_data.claims);
    
    Ok(next.run(req).await)
}
```

### Permission Check Middleware

```rust
pub fn require_permission(permission: &'static str) -> impl Fn(Request, Next) -> impl Future<Output = Result<Response, StatusCode>> {
    move |mut req: Request, next: Next| async move {
        // Get claims from request extensions
        let claims = req.extensions()
            .get::<AdminClaims>()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        // Check if user has required permission
        if !claims.permissions.contains(&permission.to_string()) {
            return Err(StatusCode::FORBIDDEN);
        }
        
        Ok(next.run(req).await)
    }
}
```

### Airline Scope Middleware

```rust
pub async fn airline_scope_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = req.extensions()
        .get::<AdminClaims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract airline_id from path
    let path = req.uri().path();
    let airline_id = extract_airline_id_from_path(path)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Super admins can access any airline
    if claims.role == "SUPER_ADMIN" {
        return Ok(next.run(req).await);
    }
    
    // Regular admins can only access their own airline
    if claims.airline_id != Some(airline_id) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

---

## Router Setup

```rust
use axum::{
    Router,
    routing::{get, post, put, delete, patch},
    middleware,
};

pub fn admin_routes(state: AppState) -> Router {
    Router::new()
        // Airline Management
        .route("/airlines", get(list_airlines).post(create_airline))
        .route("/airlines/:id", get(get_airline).put(update_airline).delete(delete_airline))
        
        // Product Management (with airline scope)
        .route("/airlines/:airline_id/products", get(list_products).post(create_product))
        .route("/products/:id", get(get_product).put(update_product).delete(delete_product))
        .route("/airlines/:airline_id/products/bulk", post(bulk_import_products))
        .route_layer(middleware::from_fn(require_permission("manage_products")))
        
        // Pricing Rules (with airline scope)
        .route("/airlines/:airline_id/pricing-rules", get(list_pricing_rules).post(create_pricing_rule))
        .route("/pricing-rules/:id", get(get_pricing_rule).put(update_pricing_rule).delete(delete_pricing_rule))
        .route("/pricing-rules/:id/test", post(test_pricing_rule))
        .route_layer(middleware::from_fn(require_permission("manage_pricing")))
        
        // Bundle Templates
        .route("/airlines/:airline_id/bundles", get(list_bundles).post(create_bundle))
        .route("/bundles/:id", get(get_bundle).put(update_bundle).delete(delete_bundle))
        .route("/bundles/:id/test", post(test_bundle))
        .route_layer(middleware::from_fn(require_permission("manage_bundles")))
        
        // Flight Management
        .route("/airlines/:airline_id/flights", get(list_flights).post(create_flight))
        .route("/flights/:id", get(get_flight).put(update_flight).delete(delete_flight))
        .route("/airlines/:airline_id/flights/bulk", post(bulk_import_flights))
        .route_layer(middleware::from_fn(require_permission("manage_flights")))
        
        // Analytics (read-only)
        .route("/airlines/:airline_id/analytics/revenue", get(get_revenue_analytics))
        .route("/airlines/:airline_id/analytics/conversion", get(get_conversion_analytics))
        .route("/airlines/:airline_id/reports/orders", get(get_orders_report))
        .route_layer(middleware::from_fn(require_permission("view_analytics")))
        
        // Apply admin auth to all routes
        .layer(middleware::from_fn_with_state(state.clone(), admin_auth_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), airline_scope_middleware))
        .with_state(state)
}

pub fn customer_routes(state: AppState) -> Router {
    Router::new()
        // Public endpoints (no auth)
        .route("/auth/login", post(customer_login))
        .route("/auth/register", post(customer_register))
        
        // Protected endpoints (customer auth)
        .route("/offers/search", post(search_offers))
        .route("/offers/:id/accept", post(accept_offer))
        .route("/orders/:id", get(get_order))
        .route("/orders/:id/pay", post(pay_order))
        .route("/orders/:id/customize", post(customize_order))
        .layer(middleware::from_fn_with_state(state.clone(), customer_auth_middleware))
        .with_state(state)
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/v1", customer_routes(state.clone()))
        .nest("/v1/admin", admin_routes(state.clone()))
}
```

---

## Security Best Practices

### 1. Token Expiry

- **Customer tokens**: 1 hour (3600 seconds)
- **Admin tokens**: 2 hours (7200 seconds)
- **Refresh tokens**: 7 days (for customers only)

### 2. Rate Limiting

```rust
// Customer endpoints: 100 req/min per IP
// Admin endpoints: 1000 req/min per user
```

### 3. CORS

```rust
// Customer API: Allow all origins (public)
// Admin API: Whitelist specific domains only
let admin_cors = CorsLayer::new()
    .allow_origin("https://admin.altis.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

### 4. Audit Logging

All admin actions are logged:

```rust
async fn log_admin_action(
    admin_id: Uuid,
    airline_id: Uuid,
    action: &str,
    resource: &str,
    changes: serde_json::Value,
) {
    db.execute(
        "INSERT INTO rule_audit_log (airline_id, action, changed_by, changes)
         VALUES ($1, $2, $3, $4)",
        &[&airline_id, &action, &admin_id.to_string(), &changes]
    ).await;
}
```

---

## Summary

| Aspect | Customer API (`/v1/*`) | Admin API (`/v1/admin/*`) |
|--------|------------------------|---------------------------|
| **Authentication** | Customer JWT | Admin JWT with permissions |
| **Rate Limit** | 100 req/min per IP | 1000 req/min per user |
| **CORS** | Allow all origins | Whitelist specific domains |
| **Token Expiry** | 1 hour | 2 hours |
| **Scope** | User's own orders | Airline-specific resources |
| **Audit Logging** | No | Yes (all actions logged) |
