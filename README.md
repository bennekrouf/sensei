# APICheck

A tool that uses LLM to match user inputs with predefined API endpoints based on semantic understanding.

## Prerequisites

- Rust (latest stable version)
- For Ollama mode: Ollama running locally with required models
- For Claude mode: Claude API key in `.env` file

## Installation

1. Clone and build:
```bash
git clone git@github.com:bennekrouf/apicheck.git
cd apicheck
cargo install --path .
```

2. Configure your environment:
   
   **For Ollama:**
   - Install Ollama from https://ollama.ai/
   - Ensure Ollama is running: `ollama serve`
   - Pull required models:
     ```bash
     ollama pull llama2
     ollama pull deepseek-r1:8b
     # or other models as specified in your config.yaml
     ```
   
   **For Claude:**
   - Get a Claude API key from https://www.anthropic.com/
   - Create a `.env` file in the project root:
     ```
     CLAUDE_API_KEY=your_api_key_here
     ```

3. For local testing generate certificates:
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Usage

After installation, you can use `apicheck` in two ways:

### CLI Mode
```bash
# Analyze a sentence with Ollama
apicheck --provider ollama "schedule a meeting tomorrow at 2pm with John"

# Analyze a sentence with Claude
apicheck --provider claude "schedule a meeting tomorrow at 2pm with John"

# Show help
apicheck --help
```

⚠️ **IMPORTANT**: The `--provider` argument is required and must be one of:
- `ollama` - Use local Ollama instance (default host: localhost:11434)
- `claude` - Use Claude API (requires API key in .env file)

### gRPC Server Mode
```bash
# Start the gRPC server with Ollama
apicheck --provider ollama

# Start the gRPC server with Claude
apicheck --provider claude
```

### Error Troubleshooting

If you see this error:
```
error: the following required arguments were not provided:
  --provider <PROVIDER>
```

Make sure to specify either `ollama` or `claude`:
```bash
apicheck --provider ollama
# or
apicheck --provider claude
```

## Configuration

1. Create a `endpoints.yaml` file in your working directory with your endpoint definitions:

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

2. Update your `config.yaml` to specify provider-specific model names:

```yaml
# Model configurations with provider-specific model names
models:
  sentence_to_json:
    ollama: "llama2"
    claude: "claude-3-7-sonnet-20250219"
    temperature: 0.1
    max_tokens: 1000
  find_endpoint:
    ollama: "deepseek-r1:8b"
    claude: "claude-3-7-sonnet-20250219"
    temperature: 0.1
    max_tokens: 500
  semantic_match:
    ollama: "deepseek-r1:8b"
    claude: "claude-3-7-sonnet-20250219"
    temperature: 0.1
    max_tokens: 500

# Provider configurations
providers:
  ollama:
    enabled: true
    host: "http://localhost:11434"
  claude:
    enabled: false  # Will be overridden by CLI flag
    api_key: ""     # Will be loaded from .env
```

## License

This project is licensed under the MIT License - see the LICENSE file for details
