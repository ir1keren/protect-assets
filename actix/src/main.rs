use actix_files::NamedFile;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use cookie::CookieBuilder;
use lazy_static::lazy_static;
use minijinja_autoreload::AutoReloader;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Minijinja template environment ──────────────────────────────────────────

lazy_static! {
    static ref RELOADER: AutoReloader = AutoReloader::new(|notifier| {
        let template_dir = PathBuf::from("templates");
        notifier.watch_path(&template_dir, true);
        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader(&template_dir));
        Ok(env)
    });
}

// ── Structs ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

// ── Template helper ───────────────────────────────────────────────────────────

fn render_template(name: &str) -> HttpResponse {
    let env = match RELOADER.acquire_env() {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Template env error: {err}");
            return HttpResponse::InternalServerError().body("Template engine error");
        }
    };

    let tmpl = match env.get_template(name) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Template not found '{name}': {err}");
            return HttpResponse::InternalServerError().body("Template not found");
        }
    };

    match tmpl.render(minijinja::context!()) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(err) => {
            eprintln!("Template render error: {err}");
            HttpResponse::InternalServerError().body("Render error")
        }
    }
}

// ── Cookie helper ─────────────────────────────────────────────────────────────

fn has_session(req: &HttpRequest) -> bool {
    req.headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').any(|p| p.trim().starts_with("session=")))
        .unwrap_or(false)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /login
#[get("/login")]
async fn login_page(req: HttpRequest) -> HttpResponse {
    if has_session(&req) {
        render_template("logged-in.html")
    } else {
        render_template("login.html")
    }
}

/// POST /api/login
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

/// GET /logout  – expire session cookie, redirect to Referer or /login
#[get("/logout")]
async fn logout(req: HttpRequest) -> HttpResponse {
    let expired_cookie = CookieBuilder::new("session", "")
        .path("/")
        .http_only(true)
        .max_age(cookie::time::Duration::seconds(0))
        .build();

    let redirect_to = req
        .headers()
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/login")
        .to_string();

    HttpResponse::Found()
        .append_header(("Location", redirect_to))
        .append_header(("Set-Cookie", expired_cookie.to_string()))
        .finish()
}

/// GET /protected/* – serve files under public/protected only when logged in
async fn get_protected(req: HttpRequest) -> impl Responder {
    let is_logged_in = has_session(&req);

    let mut target_path = std::env::current_dir().unwrap().join("public");
    for p in req.path().split('/') {
        if !p.is_empty() {
            target_path.push(p);
        }
    }

    if target_path.is_file() {
        if is_logged_in {
            match NamedFile::open(&target_path) {
                Ok(f) => f.into_response(&req),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        } else {
            render_template("login.html")
        }
    } else {
        render_template("404.html")
    }
}

/// Catch-all 404 handler
async fn not_found() -> HttpResponse {
    render_template("404.html")
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting actix server on http://127.0.0.1:8080");

    HttpServer::new(|| {
        let protected_scope =
            web::scope("/protected").default_service(web::route().to(get_protected));

        let static_files = actix_files::Files::new(
            "/assets",
            std::env::current_dir().unwrap().join("public/assets"),
        );

        App::new()
            .service(login_page)
            .service(api_login)
            .service(logout)
            .service(static_files)
            .service(protected_scope)
            .default_service(web::route().to(not_found))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
