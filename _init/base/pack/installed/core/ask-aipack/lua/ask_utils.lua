
function template() 
  return [[
```toml
#!meta parametric block

model_aliases = {pro = "claude-sonnet-4-20250514", high = "gpt-5-high", low = "gpt-5-mini-low", cheap = "gpt-5-nano-low", fast = "gemini-2.0-flash"}

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