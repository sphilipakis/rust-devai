
function template() 
  return [[
```toml
#!meta parametric block
# model = "o3-mini-low"
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