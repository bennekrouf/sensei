# Semantic Endpoint Matcher

A Rust-based tool that uses LLM to match user inputs with predefined API endpoints based on semantic understanding.

## Features

- Semantic matching using LLM (deepseek-r1:8b)
- YAML-based configuration for endpoints
- Robust error handling and logging using tracing
- Support for complex endpoint matching with parameters

## Prerequisites

- Rust (latest stable version)
- Ollama running locally with deepseek-r1:8b model


## Installation

1. Clone the repository:
```bash
git clone git@github.com:bennekrouf/semantic.git
cd semantic
```

2. Build the project:
```bash
cargo build --release
```

For local testing generate certificates :
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Configuration

Create a `endpoints.yaml` file in the project root with your endpoint definitions:

```yaml
endpoints:
  - id: schedule_meeting
    text: schedule meeting
    description: Schedule a meeting with specified participants
    parameters:
      - name: time
        description: Meeting time and date
        required: true
      - name: participants
        description: List of attendees
        required: true
  - id: ....
```

## Usage

Run the example usage to test endpoint matching:

```bash
cargo run --release
```

Example input:
```
schedule a meeting tomorrow at 2pm for 1 hour with Bill MacBride to discuss project status
```

The matcher will:
1. Parse the input
2. Compare it semantically with available endpoints
3. Return the best matching endpoint with its parameters

## License

This project is licensed under the MIT License - see the LICENSE file for details
