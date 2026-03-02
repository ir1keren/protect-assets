# Securing Static Assets in Rust: A Guide with Actix-Web and Ntex

When building web applications, developers often focus primarily on securing their API endpoints and database connections. However, a frequently overlooked aspect of web security is the protection of static assets. 

If your CSS, JavaScript, images, or configuration files contain sensitive business logic, proprietary algorithms, or paid digital content, leaving them completely public can expose your application to significant vulnerabilities and theft by web scrapers.

In this post, we'll explore a powerful yet simple pattern for securing static assets using two of Rust's most prominent web frameworks: **Actix-Web** and **Ntex**.

## The Problem with Public Assets

By default, static file servers treat everything as public. While this is perfect for your landing page CSS or public logos, it becomes an issue for applications with private dashboards or premium content. 

If a malicious actor or a web scraper discovers the URL of your raw assets (e.g., `https://your-app.com/protected/premium-widget.js`), they can download the file directly, completely bypassing your application's frontend login flow. 

## The Solution: Scoped Authentication

The solution is to intercept requests to static files and validate the user's session before reading the file from the disk. We can achieve this elegantly in both Actix and Ntex using the `scope()` function combined with the `NamedFile` struct.

### 1. Setting up the Project

First, we need to set up the project. We'll use Actix-Web for this example.
``actix/Cargo.toml`` file:
```toml
[dependencies]
actix-web = "4"
actix-files = "0.6"
cookie = "0.18"
lazy_static = "1.5"
minijinja = { version = "2.9", features = [
    "multi_template",
    "builtins",
    "json",
    "urlencode",
    "loader",
] }
minijinja-autoreload = "2.9.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### 2. Cookie-Based Session Validation

First, we need a way to determine if a request comes from an authenticated user. For simplicity, we'll look for a specific session cookie. 
Login logic:
```rust
#[post("/api/login")]
async fn api_login(body: web::Json<LoginRequest>) -> HttpResponse {
    if body.username == "admin" && body.password == "admin" {
        let session_cookie = CookieBuilder::new("session", body.username.clone())
            .path("/")
            .http_only(true)
            .build();

        HttpResponse::Ok()
            .append_header(("Set-Cookie", session_cookie.to_string()))
            .json(ApiResponse {
                success: true,
                message: "Login successful".to_string(),
            })
    } else {
        HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Invalid credentials".to_string(),
        })
    }
}
```
Javascript login logic:
```javascript
async function login() {
    const response = await fetch('/api/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            username: 'admin',
            password: 'admin',
        }),
    });

    const data = await response.json();
    if (data.success) {
        // Redirect to the protected page
        window.location.href = '/protected';
    } else {
        alert(data.message);
    }
}
```
Username and password are hard-coded for simplicity. In a production environment, you should use a database or an external authentication provider.
*Note: While our example uses simple cookie validation, this pattern is designed to be easily extended to check a database for a valid session token, validate a JWT, or integrate with an external auth provider.*

When the user logs in successfully, we issue a secure, HTTP-only cookie. When they log out, we expire that cookie.

### 2. Using `scope()` to Group Protected Routes

Instead of attaching authentication logic to every single endpoint, we can use the exact same functionality available in both Actix-Web and Ntex: the `scope()` function. 

By defining a scope (for example, `/protected`), we tell the framework that any request starting with this path should be handled by our custom logic.

```rust
// Both Actix and Ntex support grouping logic under a scope
App::new()
    .service(
        scope("/protected").default_service(route().to(get_protected))
    )
```

### 3. Serving the File with `NamedFile`

Inside our handler (`get_protected`), we first check for the presence of the session cookie. 

If the user is authenticated, we dynamically construct the file path and use the `NamedFile` struct (available in both `actix-files` and `ntex-files`) to efficiently stream the asset from the disk to the client.

If they aren't authenticated, we simply redirect them to the login page (or return an unauthorized error).

```rust
// ── Cookie helper ─────────────────────────────────────────────────────────────

fn has_session(req: &HttpRequest) -> bool {
    req.headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').any(|p| p.trim().starts_with("session=")))
        .unwrap_or(false)
}

/// GET /protected/* – serve files under public/protected only when logged in
async fn get_protected(req: HttpRequest) -> impl Responder {
    let is_logged_in = has_session(&req);

    // ./public directory is the root of static files
    // so it must be prepend with public/ prefix
    let mut target_path = std::env::current_dir().unwrap().join("public");
    for p in req.path().split('/') {
        if !p.is_empty() {
            target_path.push(p);
        }
    }

    // check if file exists
    if target_path.is_file() {
        if is_logged_in {
            // user is authenticated, serve the requested asset
            match NamedFile::open(&target_path) {
                Ok(f) => f.into_response(&req),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        } else {
            // user is not authenticated, redirect to login page
            render_template("login.html")
        }
    } else {
        // file does not exist, render 404 page
        render_template("404.html")
    }
}
```

## Why This Pattern Works So Well

1. **Framework Agnostic Concept**: Because Actix and Ntex share a common architectural lineage, this exact same pattern and almost identical code work across both frameworks.
2. **Efficiency**: `NamedFile` doesn't just read the file into memory; it handles MIME type guessing, ETag generation, and efficient streaming under the hood. 
3. **Security**: Bad actors and scrapers attempting to hit `/protected/antek-asing.js` directly via curl or automated scripts will be met with a login page rather than your proprietary code.

## Conclusion

Securing static assets is an essential step when building robust web applications that handle sensitive or premium content. By leveraging Rust's powerful web frameworks and their built-in struct `NamedFile` alongside `scope()` routing, you can easily protect your files from unauthorized access.

Want to see the complete, working implementation of this pattern? Check out the [full proof-of-concept repository](https://github.com/ir1keren/protect-assets), which includes full implementations for both Actix and Ntex, complete with Minijinja templating and session management!
