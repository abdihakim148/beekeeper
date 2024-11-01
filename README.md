<div align="center">
<img src="logo.png">
</div>

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Contributors](https://img.shields.io/github/contributors/abdihakim148/beekeeper)
![Issues](https://img.shields.io/github/issues/abdihakim148/beekeeper)

</div>



## Overview

Beekeeper is an open-source Authentication, Authorization, and User Management system. It supports both third-party and custom OAuth2.0 and OpenID Connect (OpenIDC) implementations. Built with Rust, it leverages the hexagonal architecture to allow easy swapping of adaptors, databases, and input methods.

## Features

- **Security**: Utilizes Argon2 for password hashing and JWT for token management. Initially, we will use JWT due to the lack of good Rust libraries for PASETO. We may consider implementing our own PASETO solution in the future.
- **Flexible Architecture**: Hexagonal architecture for easy integration and adaptability.

### Features to be Implemented

#### Databases
- [x] In-Memory database support
- [ ] SQLite database support
- [ ] MongoDB database support
- [ ] DynamoDB database support
- [ ] PostgreSQL database support

#### Domain
- [ ] Third-party OAuth2.0 and OpenID Connect integration
- [ ] Own OAuth2.0 and OpenID Connect implementation
- [ ] Email validation
- [ ] Logging

#### Interface
- [x] HTTP API with Actix
- [ ] GRPC input method
- [ ] Lambda input method
- [ ] YAML configuration support
- [ ] JSON configuration support

## Contributing

Contributions are welcome! Please read the [contributing guidelines](CONTRIBUTING.md) first.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For questions or suggestions, please open an issue or contact the maintainers.

---
