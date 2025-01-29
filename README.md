# Semantic Endpoint Matcher

A Rust-based tool that uses Large Language Models to match user inputs with predefined API endpoints based on semantic understanding.

## Features

- Semantic matching using LLM (deepseek-r1:8b)
- YAML-based configuration for endpoints
- Robust error handling and logging using tracing
- Support for complex endpoint matching with parameters
- Substring matching fallback for reliable endpoint detection

## Prerequisites

- Rust (latest stable version)
- Ollama running locally with deepseek-r1:8b model
- Cargo and its dependencies

## Installation

1. Clone the repository:
```bash
git clone git@github.com:bennekrouf/semantic.git
cd semantic-endpoint-matcher
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

Create a `config.yaml` file in the project root with your endpoint definitions:

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
```

## Usage

Run the example usage to test endpoint matching:

```bash
cargo run --release
```

Example input:
```
schedule a meeting tomorrow at 2pm for 1 hour with Salem Mejid to discuss project status
```

The matcher will:
1. Parse the input
2. Compare it semantically with available endpoints
3. Return the best matching endpoint with its parameters

## Project Structure

```
src/
  ├── main.rs
  ├── models/
  │   └── mod.rs
  ├── call_ollama.rs
  └── extract_matched_action.rs
```

## License

This project is licensed under the MIT License - see the LICENSE file for details
