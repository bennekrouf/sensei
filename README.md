# APICheck

A tool that uses LLM to match user inputs with predefined API endpoints based on semantic understanding.

## Prerequisites

- Rust (latest stable version)
- Ollama running locally with deepseek-r1:8b model

## Installation

1. Clone and build:
```bash
git clone git@github.com:bennekrouf/apicheck.git
cd apicheck
cargo install --path .
```

2. For local testing generate certificates:
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Usage

After installation, you can use `apicheck` in two ways:

### CLI Mode
```bash
# Analyze a sentence directly
apicheck "schedule a meeting tomorrow at 2pm with John"

# Show help
apicheck --help
```

### gRPC Server Mode
```bash
# Start the gRPC server (runs when no prompt is provided)
apicheck
```

## Configuration

Create a `endpoints.yaml` file in your working directory with your endpoint definitions:

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

## License

This project is licensed under the MIT License - see the LICENSE file for details
