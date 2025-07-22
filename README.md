# MilterDecoder

A high-performance Rust implementation of a Milter protocol server for MIME email parsing and analysis.

## Overview

MilterDecoder is a Milter (Mail Filter) protocol server written in Rust using Tokio for asynchronous processing. It receives email data from mail servers (like Postfix) via the Milter protocol, parses MIME structure, and outputs detailed email information including headers, body content, attachments, and encoding details.

## Features

- **Asynchronous Processing**: Built with Tokio for high-performance concurrent client handling
- **Full Milter Protocol Support**: Compatible with Postfix/Sendmail Milter protocol
- **MIME Email Parsing**: Complete MIME structure analysis using mail-parser
- **Detailed Output**: Extracts From/To/Subject/Content-Type/encoding/body/attachments
- **Japanese Timezone Support**: JST timestamp logging with chrono-tz
- **Signal Handling**: SIGHUP for config reload, SIGTERM for graceful shutdown
- **Configurable**: External configuration file for server settings
- **Debug Features**: NUL byte visualization, hex dump output for debugging

## Installation

### Prerequisites

- Rust 1.70 or later
- Tokio runtime
- Compatible mail server (Postfix, Sendmail, etc.)

### Building from Source

```bash
git clone https://github.com/disco-v8/MilterDecoder.git
cd MilterDecoder
cargo build --release
```

## Configuration

Create a `MilterDecoder.conf` file in the project root:

```
Listen [::]:8898
Client_timeout 30
```

### Configuration Options

- `Listen`: Server bind address and port (supports IPv4/IPv6)
  - Format: `IP:PORT` or just `PORT` (defaults to dual-stack)
  - Example: `192.168.1.100:4000` or `8898`
- `Client_timeout`: Client inactivity timeout in seconds

## Usage

### Starting the Server

```bash
./target/release/milter_decoder
```

### Postfix Integration

Add to `/etc/postfix/main.cf`:

```
smtpd_milters = inet:localhost:8898
non_smtpd_milters = inet:localhost:8898
milter_default_action = accept
```

Restart Postfix:

```bash
sudo systemctl restart postfix
```

### Signal Handling

- **SIGHUP**: Reload configuration file
- **SIGTERM**: Graceful shutdown

```bash
# Reload configuration
kill -HUP $(pidof milter_decoder)

# Graceful shutdown
kill -TERM $(pidof milter_decoder)
```

## Output Format

The server outputs detailed email analysis to stdout with JST timestamps:

```
[2024/07/22 15:30:45] --- BODYEOB時のメール全体 ---
[2024/07/22 15:30:45] [mail-parser] from: sender@example.com
[2024/07/22 15:30:45] [mail-parser] to: recipient@example.com
[2024/07/22 15:30:45] [mail-parser] subject: Test Email
[2024/07/22 15:30:45] [mail-parser] content-type: "text/plain; charset=utf-8"
[2024/07/22 15:30:45] [mail-parser] テキストパート数: 1
[2024/07/22 15:30:45] [mail-parser] 非テキストパート数: 0
[2024/07/22 15:30:45] 本文(1): Hello, this is a test email.
```

### Multi-part Email with Attachments

```
[2024/07/22 15:31:20] このメールはマルチパートです
[2024/07/22 15:31:20] [mail-parser] テキストパート数: 1
[2024/07/22 15:31:20] [mail-parser] 非テキストパート数: 1
[2024/07/22 15:31:20] 本文(1): Email body content
[2024/07/22 15:31:20] 非テキストパート(1): content_type="application/pdf", encoding=Base64, filename=document.pdf, size=1024 bytes
```

## Architecture

### Module Structure

- **main.rs**: Server startup, configuration management, signal handling
- **client.rs**: Per-client Milter protocol handling
- **milter.rs**: Milter command decoding and response generation
- **milter_command.rs**: Milter protocol command definitions
- **parse.rs**: MIME email parsing and output formatting
- **init.rs**: Configuration file management
- **logging.rs**: JST timestamp logging macros

### Milter Protocol Flow

1. **OPTNEG**: Protocol negotiation
2. **CONNECT**: Client connection information
3. **HELO/EHLO**: SMTP greeting
4. **DATA**: Macro information
5. **HEADER**: Email headers (multiple)
6. **BODY**: Email body content (multiple chunks)
7. **BODYEOB**: End of body - triggers email parsing and output

## Dependencies

- [tokio](https://tokio.rs/): Asynchronous runtime
- [mail-parser](https://crates.io/crates/mail-parser): MIME email parsing
- [chrono](https://crates.io/crates/chrono): Date and time handling
- [chrono-tz](https://crates.io/crates/chrono-tz): Timezone support
- [lazy_static](https://crates.io/crates/lazy_static): Global static variables

## Development

### Running in Development Mode

```bash
cargo run
```

### Testing with Sample Email

You can test the server by sending emails through a configured Postfix instance or using telnet to send raw SMTP commands.

### Debug Features

- NUL byte visualization: `\0` bytes are displayed as `<NUL>`
- Hex dump output for unknown commands
- Detailed protocol command logging
- Error handling with descriptive messages

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For issues, questions, or contributions, please open an issue on GitHub.

## Changelog

### v0.1.0
- Initial release
- Basic Milter protocol implementation
- MIME email parsing and output
- Configuration file support
- Signal handling
- JST timestamp logging
