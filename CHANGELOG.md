# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-07-23

### Improved
- Enhanced email body output logic to support both TEXT and HTML content simultaneously
- Excluded multipart parent parts (like multipart/alternative) from body output to avoid unnecessary "decode error" messages
- Added content type filtering to only output text/plain and text/html parts
- Significantly improved code readability with comprehensive comments (one comment per line minimum)
- Better error handling for various text part subtypes

### Technical Improvements
- Refined `parse.rs` body extraction logic using `body_text()` and `body_html()` methods
- Implemented proper multipart parent part detection using `c_type.eq_ignore_ascii_case("multipart")`
- Added detailed inline documentation explaining RFC compliance, encoding handling, and processing logic
- Enhanced type safety with proper `Option` handling for content subtypes

## [0.1.0] - 2025-07-22

### Added
- Initial release of MilterDecoder
- Full Milter protocol implementation compatible with Postfix/Sendmail
- Asynchronous TCP server using Tokio runtime
- MIME email parsing with mail-parser crate
- Comprehensive email analysis and output:
  - From/To/Subject extraction
  - Content-Type and encoding detection
  - Multi-part email support
  - Attachment detection with filename extraction
  - Text/non-text part classification
- JST timestamp logging with chrono-tz
- Configuration file support (`MilterDecoder.conf`)
- Signal handling:
  - SIGHUP for configuration reload
  - SIGTERM for graceful shutdown
- Debug features:
  - NUL byte visualization
  - Hex dump output for unknown commands
  - Detailed protocol logging
- Error handling and timeout management
- IPv4/IPv6 dual-stack support

### Technical Features
- Modular architecture with clear separation of concerns:
  - `main.rs`: Server startup and management
  - `client.rs`: Per-client Milter protocol handling
  - `milter.rs`: Milter command processing
  - `milter_command.rs`: Protocol definitions
  - `parse.rs`: Email parsing and analysis
  - `init.rs`: Configuration management
  - `logging.rs`: Timestamp logging utilities
- Comprehensive documentation and comments
- Rust 2021 edition compatibility
- MIT license

### Dependencies
- tokio 1.38 (async runtime)
- mail-parser 0.11 (MIME parsing)
- chrono 0.4 (date/time handling)
- chrono-tz 0.8 (timezone support)
- lazy_static 1.5.0 (global variables)

[Unreleased]: https://github.com/disco-v8/MilterDecoder/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/disco-v8/MilterDecoder/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/disco-v8/MilterDecoder/releases/tag/v0.1.0
