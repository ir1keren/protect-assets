use lazy_static::lazy_static;
use minijinja_autoreload::AutoReloader;

use ntex::http::header::{self, HeaderValue};
use ntex::http::{ResponseBuilder, StatusCode};
use ntex::web::{self, HttpRequest, HttpResponse, Responder, route, scope};
use ntex_files::{Files, NamedFile};
use ntex_remove_trailing_slash::RemoveTrailingSlash;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Minijinja template environment ──────────────────────────────────────────

lazy_static! {
    static ref RELOADER: AutoReloader = AutoReloader::new(|notifier| {
        let template_dir = PathBuf::from("ntex/templates");
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

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn get_protected(http_req: HttpRequest) -> impl Responder {
    let is_logged_in = http_req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|cookie_str| {
            cookie_str
                .split(';')
                .any(|part| part.trim().starts_with("session="))
        })
        .unwrap_or(false);

    let mut target_path = std::env::current_dir().unwrap().join("public");
    let path = http_req.path();

    for p in path.split('/') {
        if p.is_empty() {
            continue;
        }

        target_path.push(p);
    }

    if target_path.is_file() {
        if is_logged_in {
            let result = match NamedFile::open(&target_path) {
                Err(e) => {
                    return ResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(e.to_string());
                }
                Ok(res) => res,
            };

            result.into_response(&http_req)
        } else {
            render_template("login.html")
        }
    } else {
        render_template("404.html")
    }
}

/// Render a minijinja template by name, returning an HTML response.
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

/// GET /login  – show login form, or "already logged in" if session cookie is present
async fn login_page(req: HttpRequest) -> HttpResponse {
    let is_logged_in = req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|cookie_str| {
            cookie_str
                .split(';')
                .any(|part| part.trim().starts_with("session="))
        })
        .unwrap_or(false);

    if is_logged_in {
        render_template("logged-in.html")
    } else {
        render_template("login.html")
    }
}

/// POST /api/login  – validate credentials and set session cookie
async fn api_login(_req: HttpRequest, body: web::types::Json<LoginRequest>) -> HttpResponse {
    if body.username == "admin" && body.password == "admin" {
        let session_cookie = cookie::CookieBuilder::new("session", body.username.clone())
            .path("/")
            .http_only(true)
            .build();

        let cookie_str = session_cookie.to_string();

        let mut response = HttpResponse::Ok().json(&ApiResponse {
            success: true,
            message: "Login successful".to_string(),
        });

        if let Ok(cookie_value) = HeaderValue::from_str(&cookie_str) {
            response
                .headers_mut()
                .insert(header::SET_COOKIE, cookie_value);
        }

        response
    } else {
        HttpResponse::Unauthorized().json(&ApiResponse {
            success: false,
            message: "Invalid credentials".to_string(),
        })
    }
}

/// GET /logout  – expire the session cookie and redirect to /login
async fn logout(req: HttpRequest) -> HttpResponse {
    let expired_cookie = cookie::CookieBuilder::new("session", "")
        .path("/")
        .http_only(true)
        .max_age(cookie::time::Duration::seconds(0))
        .build();

    let cookie_str = expired_cookie.to_string();

    let redirect_to = req
        .headers()
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/login");

    let mut response = HttpResponse::Found()
        .header(header::LOCATION, redirect_to)
        .finish();

    if let Ok(cookie_value) = HeaderValue::from_str(&cookie_str) {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie_value);
    }

    response
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!("Starting ntex server on http://127.0.0.1:8080");

    web::HttpServer::new(|| {
        web::App::new()
            .wrap(RemoveTrailingSlash::default())
            .route("/login", web::get().to(login_page))
            .route("/api/login", web::post().to(api_login))
            .route("/logout", web::get().to(logout))
            .service(Files::new(
                "/assets",
                std::env::current_dir().unwrap().join("public/assets"),
            ))
            .service(scope("/protected").default_service(route().to(get_protected)))
            .default_service(web::route().to(|| async { render_template("404.html") }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
