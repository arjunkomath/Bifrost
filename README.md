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

## Contribution

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests to enhance the functionality and usability of this microservice.

## License

This project is licensed under the MIT License.
