# Bifrost

[![Build](https://github.com/arjunkomath/Bifrost/actions/workflows/build.yml/badge.svg)](https://github.com/arjunkomath/Bifrost/actions/workflows/build.yml)

This microservice provides a robust solution for storing data securely using encryptiong, leveraging the powerful capabilities of the Rust Ring crate.

## Endpoints
```
POST /v1/secret/{key}
create a secret

GET /v1/secret/{key}
get secret using the key

DELETE /v1/secret/{key}
delete secret

GET /health
health check
```

Reference for Authenticated Encryption with Associated Data (AEAD): https://web3developer.io/authenticated-encryption-in-rust-using-ring/

## API Specification

An OpenAPI specification is available in [`openapi.yaml`](./openapi.yaml) for generating client SDKs.

## Self Hosting

Self-hosting with your own database ensures your encrypted data never leaves your infrastructure.

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `API_KEY` | Yes | API key for authenticating requests (sent via `x-api-key` header) |
| `ENCRYPTION_KEY` | Yes | Must be exactly 32 characters for AES-256-GCM encryption (generate with `openssl rand -base64 24`) |
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `PORT` | No | Server port (defaults to `8080`) |

### Database

Bifrost uses PostgreSQL. The required table is automatically created on startup:

```sql
CREATE TABLE IF NOT EXISTS vault (
    key TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    created_at TEXT NOT NULL
)
```

### Docker

Build and run using the provided Dockerfile:

```bash
docker build -f build.Dockerfile -t bifrost .

docker run -p 8080:8080 \
  -e API_KEY="your-api-key" \
  -e ENCRYPTION_KEY="your-32-character-encryption-key" \
  -e DATABASE_URL="postgresql://user:password@host:port/database" \
  bifrost
```

## Contribution

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests to enhance the functionality and usability of this microservice.

## License

This project is licensed under the MIT License.
