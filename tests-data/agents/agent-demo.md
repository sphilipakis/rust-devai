# Config

```toml
[genai]
model = "test_model_for_demo"

[runtime]
items_concurrency = 8
```

# Data

```rhai
// Some scripts that load the data
// Will have access to the command line args after command name

let src_builder_file = file::load("./src/client/builder.rs");
// src_builder_file.name: "builder.rs"
// src_builder_file.path: "./src/client/builder.rs"
// src_builder_file.content: ".... content of the file ...."

return "hello"
```

# Instruction

Some paragraph for instruction

- One 
- Two
* Thre

1. four
2. five

- . some instruction, will support handlebars in this block (first system content) ..
* stuff

# message: System

- . possible extra system message (will support handlebars) ...

# message: User

- . possible extra user message (will support handlebars) ...

# Output

```rhai
/// Optional output processing.
```

# After All

```rhai
/// Optional after all processing.
```