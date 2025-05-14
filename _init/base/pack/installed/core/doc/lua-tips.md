## Lua Lang

```lua

-- Check if string is empty (empty or only white characters)
if not content:find("%S") then
  print("Is empty")
end


```
## Best practice to proces multiple files

Let's say we want to summarize many files, one good practice is to let the each input process each file and append it to a shared file. 

- Use `# Before All` for the common properties (will be ran onces for all inputs)
- Use `# Data` and `# Output` to prep prompt data and process ai reponse (this will be ran for each input, and can be parallelise with `input_concurrency`)
- Use `# After All` for final messages

## How pass a common value to many inputs

We can use the `# Before All` stage to return a common data that will be used accross `# Data`, `# Output`, and even `# After All`

for example

````md

# Before All

```lua
return {
    summary_path: "doc/summary.md`
}
```

# Output

```lua
local content = ai_response.content

aip.file.save(before_all.summary_path, content)

```

````

## Skip if a string only has whitespace

```lua
-- If the string contains only whitespace, treat it as empty and skip.
-- `%S` matches any non-whitespace character.
-- `find` returns 
--    - the start and end indices if a match is found;
--    - otherwise, it returns nil. Thus, `not ...` evaluates to true.
if not content:find("%S") then
    return aipack.skip("Empty file - skipping for now. Start writing and do a redo.")
end
```
