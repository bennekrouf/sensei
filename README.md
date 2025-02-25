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

After installation, you can use `apicheck` in several ways:

### CLI Mode with Local Endpoints
```bash
# Analyze a sentence with Ollama
apicheck --provider ollama "schedule a meeting tomorrow at 2pm with John"

# Analyze a sentence with Claude
apicheck --provider claude "schedule a meeting tomorrow at 2pm with John"
```

### CLI Mode with Remote Endpoints
```bash
# Use a remote gRPC endpoint service for fetching endpoints
apicheck --provider claude --api http://example.com:50053 "schedule a meeting tomorrow at 2pm with John"

# With email authentication
apicheck --provider claude --api http://example.com:50053 --email user@example.com "schedule a meeting tomorrow at 2pm with John"
```

### gRPC Server Mode
```bash
# Start the gRPC server with Ollama (using local endpoints)
apicheck --provider ollama

# Start the gRPC server with Claude (using local endpoints)
apicheck --provider claude

# Start the gRPC server with Claude (using remote endpoints)
apicheck --provider claude --api http://example.com:50053
```

### Help
```bash
# Show help
apicheck --help
```

⚠️ **IMPORTANT**: The `--provider` argument is required and must be one of:
- `ollama` - Use local Ollama instance (default host: localhost:11434)
- `claude` - Use Claude API (requires API key in .env file)

## Remote Endpoint Service

APICheck can now use a remote gRPC endpoint service to fetch the list of endpoints instead of using a local file. The remote service must implement the following gRPC service:

```protobuf
syntax = "proto3";
package endpoint;

service EndpointService {
    rpc GetDefaultEndpoints (GetEndpointsRequest) returns (stream GetEndpointsResponse);
    rpc UploadEndpoints (UploadEndpointsRequest) returns (UploadEndpointsResponse);
}

message GetEndpointsRequest {
    string email = 1;
}

message Parameter {
    string name = 1;
    string description = 2;
    bool required = 3;
    repeated string alternatives = 4;
}

message Endpoint {
    string id = 1;
    string text = 2;
    string description = 3;
    repeated Parameter parameters = 4;
    string verb = 5;
}

message GetEndpointsResponse {
    repeated Endpoint endpoints = 1;
}

message UploadEndpointsRequest {
    string email = 1;
    bytes file_content = 2;
    string file_name = 3;
}

message UploadEndpointsResponse {
    bool success = 1;
    string message = 2;
    int32 imported_count = 3;
}
```

### Client Authentication

When using a remote endpoint service, you can specify an email address for authentication:

```bash
apicheck --provider claude --api http://example.com:50053 --email user@example.com "analyze this sentence"
```

For gRPC clients connecting to the APICheck server, you can provide the email in the metadata:

```
email: user@example.com
```

## Sequence of Operation

1. When using the `--api` flag, APICheck will attempt to fetch endpoints from the specified remote service
2. If the remote service is unavailable, it will fall back to the local `endpoints.yaml` file
3. The email will be passed to the remote service for authentication (if provided)
4. The rest of the analysis works the same way as with local endpoints

## Configuration

1. Create a `endpoints.yaml` file in your working directory with your endpoint definitions (used as fallback):

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
