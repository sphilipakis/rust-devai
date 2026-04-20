# openai

- Make sure all models are all lowercase
- for OpenAI models
  - Only take the "standards" pricing,
  - set `cache_write: null` when not defined.
  - ignore the legacy models
  - Do not add the gpt-image-1 or audio models
  - for the `recent`
    - Only the latest 5.4 models `recent: true`. 
    - All others, `recent: false` (codex-mini-latest is alwys `recent: false`)
  - ignore the lines have have dates like text, like `2024-05-13`
  - only take the Text tokens section, ignore the other sections.
  - there are not cash write, so, only input and cached when present, and then the output.
  - Only take the price and names for the lesser context tokens, when there is multiple (do not have the `(.... context ..)` in the name)

# fireworks

- Make sure all models are all lowercase
- For Fireworks models:
  - For the Fireworks models, the model name is after the `/models/fireworks/`. So, `/models/fireworks/qwen3-coder-480b-a35b-instruct` the model name will be `qwen3-coder-480b-a35b-instruct`
  - For the Fireworks models, when only one price, it means same price for input and output.
  - For the fireworks models, Look at the version number for each model type, and make the one with the highest number `recent: true` (if only version for that model type, then, `recent: true`) otherwise, false. 

# together

- Make sure all models are all lowercase
- For Together Models:
  - For together models only add models that have pricing.
  - For together models, when one price, means it is the same for input / output
  - For the together models, the model name is what is in the model name section. make sure to use that name that match the one of the pricing.
  - The together model names are in the <h1>Names</h1> section.
  - Set the flag `recent: true` only for the model names that match the `<h1>recent</h1>` list, and all the other should be `recent: false`
  - Also, only extract the models about Llama, qwen, deepseek, glm, gemma, mixtral, mistral, and openai.

# anthropic

- Make sure all models are all lowercase
- For Anthropic
  - Here are the correct patterns to usefor the anthropic models (use use `-` rather than `.` in the model names)
    - `claude-opus-4-5`, ...,  `claude-sonnet-4-5`, `claude-haiku-4-5`
    - `claude-opus-4-1`, `claude-sonnet-4`,
    - `claude-3-7-sonnet`, `claude-opus-4`, `claude-3-5-haiku`
  - So match the model names in the pages to those one.
  - For Anthropic Models (Claude), Make sure to have only one `recent: true` for each model type, Sonnet, Haiku, Opus. Pick the highest number for each of them to mark as `recent: true`
  - Make sure to remove / not add the "deprecated" ones
  - For Anthropic Models (Claude), the name/value is inverse. For value, and then, label of this value.
  - For Anthropic Models (Claude), when caching write value, put it in `.cache_write`, and then, the cache value goes in `.cached`
  - For Anthropic Models, cache write price, take the `5m` price
  - For Anthropic models (Claude), make sure you get the model names from the model is, and remove the `-2025...`

# gemini

- Make sure all models are all lowercase
- For gemini models,
  - For gemini models there is no `caching write`, so cache value is for the `.cached` property.
  - For Gemini models, do not add the audio or tts model.
  - For gemini models, keep the `.` in the name.
  - Remove the `-preview` suffixes of the model names
  - For gemini models the `pro`, `flash`, and `flash-lite` there can be only one `recent: true` for each of those type. Make sure to take the highest version of each. The rest is `recent: false`
  - For Gemini models, for `gemini-2.5-pro` remove the release number after pro like the `-09...` same for other gemini models. Just have the main one.
  - For the Gemini models, add new models for
    - `gemini-flash-latest` based on `gemini flash 3 preview` pricing.
    - `gemini-flash-lite-latest` based on `gemini-2.5-flash-lite` pricing
    - `gemini-pro-latest` based on `gemini pro 3` pricing.
    - Not that bothg of this latest will be `recent: false` as they ar aboused on the 2.5
  - For gemini models, only add the model that starts with `gemini...`

# groq

- Make sure all models are all lowercase
- For Groq models,
  - For Groq models there are two sections in the markdown, the pricing and the name. Make sure to take the name from the name section, and infer the "display name" to get the pricing.
  - For Groq models, the models that are both section name and price, mark them as `recent: true`, if they are not in the model names, then `recent: false`

# zai

- Make sure all models are all lowercase
- For ZAI models
  - only `glm-5..` are `recent: true` everything else is false.

