use crate::_test_support::assert_contains;
use crate::agent::Agent;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_agent_parse_simple() -> Result<()> {
	// -- Setup & Fixtures
	let content = r#"
# Data 
```lua
-- Some lua data
```
# System

System 1
```
Some system block 1
## Heading 2 System 1
```

# System 
System 2

# User

User 1

# Output

```lua
return "hello"
```
		"#;

	// -- Exec
	let agent = Agent::mock_from_content(content)?;

	// -- Check
	let parts = agent.prompt_parts();
	assert_eq!(parts.len(), 3);
	assert_contains(&parts[0].content, "System 1");
	assert_contains(&parts[0].content, "```");
	assert_contains(&parts[0].content, "Some system block 1");
	assert_contains(&parts[0].content, "## Heading 2 System 1");
	assert_contains(&parts[1].content, "System 2");
	assert_contains(&parts[2].content, "User 1");
	assert_contains(
		agent.data_script().expect("Should have data script"),
		"-- Some lua data",
	);

	Ok(())
}
