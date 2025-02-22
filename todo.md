# Code Duplication Analysis

## Structural Duplications

### 1. Parameter Structures
There are multiple similar parameter-related structures:
- `EndpointParameter` in models/mod.rs
- `Parameter` in models/mod.rs  
- `Parameter` in sentence_service.proto

These share similar fields:
```rust
// EndpointParameter
pub struct EndpointParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub alternatives: Vec<String>,
}

// Parameter
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub semantic_value: Option<String>,
}
```

Recommendation: Consider unifying these into a single parameter structure with optional fields.

### 2. Configuration Structures
Multiple configuration-related structures exist:
- `ModelsConfig` in models/config.rs
- `ConfigFile` in models/mod.rs
- `PromptConfig` in prompts/mod.rs

## Functional Duplications

### 1. File Reading Logic
Multiple instances of similar file reading code:
- `load_models_config()` in models/config.rs
- `ConfigFile::load()` in models/mod.rs
- `PromptManager::new()` in prompts/mod.rs

Example:
```rust
// In multiple places:
let config_str = tokio::fs::read_to_string("config.yaml").await?;
let config: ConfigType = serde_yaml::from_str(&config_str)?;
```

Recommendation: Create a generic configuration loading function.

### 2. JSON Processing
Similar JSON processing patterns in:
- `zero_sentence_to_json.rs`
- `fourth_match_fields.rs`

Both use similar validation and extraction patterns.

### 3. Workflow Steps
The workflow implementation has some duplication:
- `analyze_sentence.rs` and `workflow.rs` both contain workflow-related code
- Multiple implementations of similar execution patterns

## Architectural Patterns That Could Be Unified

1. Error Handling
- Multiple similar error handling patterns across files
- Consider creating a unified error type

2. Configuration Loading
- Multiple files reading YAML configurations
- Could be unified into a single configuration management system

3. Model Instance Creation
- Similar patterns for creating model instances
- Could be abstracted into factory patterns

## Recommendations

1. Create Unified Structures:
```rust
pub struct UnifiedParameter {
    pub name: String,
    pub description: String,
    pub required: Option<bool>,
    pub alternatives: Option<Vec<String>>,
    pub semantic_value: Option<String>,
}
```

2. Create Generic Configuration Loader:
```rust
async fn load_config<T: DeserializeOwned>(path: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
    let config_str = tokio::fs::read_to_string(path).await?;
    let config: T = serde_yaml::from_str(&config_str)?;
    Ok(config)
}
```

3. Unify Workflow Implementation:
- Merge workflow-related code into a single module
- Create reusable workflow components

4. Create Common JSON Processing Utilities:
- Abstract common JSON validation and extraction patterns
- Create reusable JSON processing functions

## Priority Actions

1. High Priority:
   - Unify parameter structures
   - Create generic configuration loading
   - Consolidate workflow implementation

2. Medium Priority:
   - Unify JSON processing utilities
   - Create common error handling

3. Low Priority:
   - Refactor model instance creation
   - Optimize configuration structures
