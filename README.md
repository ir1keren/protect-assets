# Protected Web Assets Demonstration

This repository serves as a proof-of-concept for securing static web assets (such as CSS and JavaScript files) behind an authentication layer. By implementing server-side session validation, this approach effectively prevents unauthorized access and limits exposure to web scrapers and unauthorized downloads.

To demonstrate the versatility of this concept, implementations are provided for two prominent Rust web frameworks: **Actix Web** and **ntex**.

## Secure Assets

The project includes two private endpoints demonstrating the protected asset serving, mapped to the following local paths:
- `./public/protected/antek-asing.css` → `http://localhost:8080/protected/antek-asing.css`
- `./public/protected/antek-asing.js`  → `http://localhost:8080/protected/antek-asing.js`

Attempting to access these URLs without an active, authenticated session will automatically redirect the client to the login page.
This can be achieved using ``scope()`` function and ``NamedFile`` struct, since both frameworks support these features.

## Key Features

- **Authentication Flow:** Complete login and logout implementations.
- **Session Management:** Secure cookie-based session handling.
- **Access Control:** Middleware/Handlers for protecting static assets.
- **Error Handling:** Custom 404 page rendering for unmapped routes.

## Getting Started

### Prerequisites
- Rust (Cargo)

### Installation & Execution

```bash
# Clone the repository
$ git clone <repository-url>

# Navigate to the project directory
$ cd protect-assets

# Run the Actix Web implementation
$ cargo run --package protect-assets-actix --bin protect-assets-actix

# OR run the ntex implementation
$ cargo run --package protect-assets-ntex --bin protect-assets-ntex
```