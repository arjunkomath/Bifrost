# Bifrost

A simple microservice for encrypting and decrypting data using the `ring` crate using the `AES_256_GCM` algorithm. This service is intended to be used as a building block for other services that require encryption and decryption of data.

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