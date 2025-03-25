
function template() 
  return [[
```toml
#!meta parametric block

model_aliases = {pro = "claude-3-7-sonnet-latest", high = "o3-mini-high", low = "o3-mini-low", cheap = "gpt-4o-mini", fast = "gemini-2.0-flash"}

model = "cheap"
```

====
> Usage:
>    - Ask your question above the ==== separator
>    - and `aip run core@ask-aipack`
>    - or **Replay** in the terminal if already running
>    - Answers will appear below, in markdown sections
>    - You can remove those lines starting with >, 
>    - they are not included in the prompt

]]
end


return {
  template = template
}