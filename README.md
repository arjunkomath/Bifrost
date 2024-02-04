# Bifrost

[![Build](https://github.com/arjunkomath/Bifrost/actions/workflows/build.yml/badge.svg)](https://github.com/arjunkomath/Bifrost/actions/workflows/build.yml)

This microservice provides a robust solution for encrypting and decrypting data with the AES_256_GCM algorithm, leveraging the powerful capabilities of the Ring crate. Designed as a foundational component, it serves as a reliable building block for integrating encryption and decryption functionalities into various services.

## Endpoints
```
PUT /v1/token
tokenize a string

GET /v1/token/{key}
get the tokenized string using the token key

GET /health
health check
```

Reference for Authenticated Encryption with Associated Data (AEAD): https://web3developer.io/authenticated-encryption-in-rust-using-ring/

## Contribution

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests to enhance the functionality and usability of this microservice.

## License

This project is licensed under the MIT License.
