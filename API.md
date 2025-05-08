# API Documentation

## Overview

This document provides details of all endpoints exposed by the semantic routing backend service. The service provides natural language processing capabilities to route user inputs to predefined API endpoints based on semantic understanding.

## Server Configuration

- **Default Host**: 0.0.0.0
- **Default Port**: 50053

## gRPC Services

### SentenceService

Handles sentence analysis and semantic routing.

```protobuf
service SentenceService {
  rpc AnalyzeSentence (SentenceRequest) returns (stream SentenceResponse) {}
}
```

#### AnalyzeSentence

Analyzes user input and matches it to predefined endpoints.

- **Request**: `SentenceRequest`
  ```protobuf
  message SentenceRequest {
    string sentence = 1;
  }
  ```

- **Response**: Stream of `SentenceResponse`
  ```protobuf
  message SentenceResponse {
    string endpoint_id = 1;
    string endpoint_description = 2;
    repeated Parameter parameters = 3;
    string json_output = 4;
  }
  
  message Parameter {
    string name = 1;
    string description = 2;
    optional string semantic_value = 3;
  }
  ```

- **Metadata**:
  - `email`: User email for authentication (required)
  - `client-id`: Optional client identifier

## External Services

### Endpoint Service (Proxy)

The backend proxies requests to an Endpoint Service to fetch available API endpoints.

- **Default Host**: localhost
- **Default Port**: 50055
- **Configuration Parameter**: `endpoint_client.default_address` in config.yaml
- **Can be overridden**: Using `--api` CLI parameter

```protobuf
service EndpointService {
  rpc GetApiGroups (GetApiGroupsRequest) returns (stream GetApiGroupsResponse);
  rpc UploadApiGroups (UploadApiGroupsRequest) returns (UploadApiGroupsResponse);
  rpc GetUserPreferences (GetUserPreferencesRequest) returns (GetUserPreferencesResponse);
  rpc UpdateUserPreferences (UpdateUserPreferencesRequest) returns (UpdateUserPreferencesResponse);
  rpc ResetUserPreferences (ResetUserPreferencesRequest) returns (ResetUserPreferencesResponse);
}
```

### AI Model Providers

The backend connects to one of the following AI model providers:

#### Claude API
- **Host**: https://api.anthropic.com/v1/messages
- **Authentication**: API key in environment variable `CLAUDE_API_KEY`
- **Enabled via**: `--provider claude` CLI parameter

#### Ollama (Local Models)
- **Default Host**: http://localhost:11434
- **Port**: Default Ollama port
- **Path**: /api/generate
- **Enabled via**: `--provider ollama` CLI parameter
- **Configuration Parameter**: `providers.ollama.host` in config.yaml

## Authentication

The gRPC API requires email-based authentication. Email must be provided in the request metadata.

## Sample Usage

### gRPC Client Example

```bash
grpcurl -plaintext \
  -d '{"sentence": "Send an email to john@example.com with subject Hello"}' \
  -H "email: user@example.com" \
  0.0.0.0:50053 \
  sentence.SentenceService/AnalyzeSentence
```

### CLI Example

```bash
api0 --provider claude --email user@example.com "Send an email to john@example.com with subject Hello"
```
