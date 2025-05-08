<div align="center">

<a href="https://crates.io/crates/aipack"><img src="https://img.shields.io/crates/v/aipack.svg" /></a>
<a href="https://github.com/jeremychone/rust-aipack"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=SioUg_N9HS0"><img alt="Static Badge" src="https://img.shields.io/badge/AIPACK_Introduction_-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# AIPACK - Run, Build, and Share AI Packs

Check out the site: https://aipack.ai for more information and links, [AIPACK News & Blog Posts](https://news.aipack.ai/archive)

Open-source Agentic Runtime to run, build, and share AI Packs.

- Supports **all** major AI providers and models.
- Efficient and small (**< 20MB**), with **zero** dependencies.
- Built in **Rust** using Lua for embedded scripting (small and efficient).
- Runs locally, completely IDE-agnostic.
- Or in the cloud—server or serverless.

<img alt="Static Badge" src="https://img.shields.io/badge/AIPACK_VIDEOS_-Video?style=flat&logo=youtube&color=%23ff0000">

- [Video: pro@coder with pro@rust10x Rust best practices](https://www.youtube.com/watch?v=rIAoSf4TWho&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigq)
- [Video: jc@coder (now pro@coder) Pack Demo ](https://www.youtube.com/watch?v=-xFd00rrfLk&list=PL7r-PXl6ZPcDBodiiTdUeCmUwsYFyDzGt)
- [Video: AIPACK Introduction](https://www.youtube.com/watch?v=SioUg_N9HS0&list=PL7r-PXl6ZPcDBodiiTdUeCmUwsYFyDzGt)
- [Video: AIPACK Playlist](https://www.youtube.com/playlist?list=PL7r-PXl6ZPcDBodiiTdUeCmUwsYFyDzGt)
- [AIPACK Substack at news.aipack.ai](https://news.aipack.ai)

## Quick Start

### Install From Binaries

Mac, Linux, Windows, ARM & x86 platforms are supported. See blow how to install the binaries. 

_(More info at [aipack.ai/doc/install](https://aipack.ai/doc/install))_

#### Mac

```sh
# Mac ARM / Apple Silicon
curl -O https://repo.aipack.ai/aip-dist/stable/latest/aarch64-apple-darwin/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup

# Mac x86
curl -O https://repo.aipack.ai/aip-dist/stable/latest/x86_64-apple-darwin/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup        
```


#### Linux

```sh
# Linux x86
curl -O https://repo.aipack.ai/aip-dist/stable/latest/x86_64-unknown-linux-gnu/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup

# Linux ARM
curl -O https://repo.aipack.ai/aip-dist/stable/latest/aarch64-unknown-linux-gnu/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup
```

#### Windows

```sh
# Windows x86
Invoke-WebRequest -Uri "https://repo.aipack.ai/aip-dist/stable/latest/x86_64-pc-windows-msvc/aip.tar.gz" -OutFile "aip.tar.gz"
tar -xvf aip.tar.gz
.\aip.exe self setup
     

# Windows ARM
Invoke-WebRequest -Uri "https://repo.aipack.ai/aip-dist/stable/latest/aarch64-pc-windows-msvc/aip.tar.gz" -OutFile "aip.tar.gz"
tar -xvf aip.tar.gz
.\aip.exe self setup
```


### Install from source

For now, installation requires building directly from source via Rust. Works on all major OSes.

- Install Rust: https://www.rust-lang.org/tools/install
- For now, install with `cargo install aipack`


### Run

```sh
# In the terminal, go to your project
cd /path/to/my/project/

# Initialize workspace .aipack/ and ~/.aipack-base
aip init

# Make sure to export the desired API key (no spaces around `=` unix convention)
export OPENAI_API_KEY="sk...."
export ANTHROPIC_API_KEY="...."
export GEMINI_API_KEY="..."
# For more keys, see below

# Check the keys you setup
aip check-keys

# To proofread your README.md (namespace: demo, pack_name: proof)
aip run demo@proof -f ./README.md

# You can just use @pack_name if there is no other pack with this name
aip run @proof -f ./README.md

# To do some code crafting (will create `_craft-code.md`)
aip run demo@craft/code

# Or run your .aip file (you can omit the .aip extension)
aip run path/to/file.aip

# This is a good agent to run to ask questions about aipack
# It can even generate aipack code
aip run core@ask-aipack
# The prompt file will be at `.aipack/.prompt/core@ask-aipack/ask-prompt.md`

```

### Thanks to

-  **[Stephane Philipakis](https://github.com/sphilipakis)**, a key [aipack](https://aipack.ai) collaborator.
-  [David Horner](https://github.com/davehorner) for adding Windows support for Open Agent (with VSCode) ([#30](https://github.com/jeremychone/rust-aipack/pull/30))
-  [Diaa Kasem](https://github.com/diaakasem) for `--non-interactive`/`--ni` (Now, in `v0.7.x` `-s` or `--single-shot`) mode ([#28](https://github.com/jeremychone/rust-aipack/pull/28))


### `pro@coder`

- You can install `pro@coder` with `aip install pro@coder`, and then
- Run it with `aip run pro@coder` or `aip run @coder` if you don't have any other `@coder` pack in a different namespace.

This is the agent I use every day for my production coding.

**IMPORTANT 1**: Make sure everything is committed before use (at least while you are learning about aipack).

**IMPORTANT 2**: Make sure to have your **API_KEY** set as an environment variable (on Mac, there is experimental keychain support).

```
OPENAI_API_KEY
ANTHROPIC_API_KEY
GEMINI_API_KEY
XAI_API_KEY
DEEPSEEK_API_KEY
GROQ_API_KEY
COHERE_API_KEY
```

## More Info

- Website: https://aipack.ai

- [AIPACK Overview Video](https://www.youtube.com/watch?v=SioUg_N9HS0)

- [Preview 'devai' intro video for v0.5](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)

- Built on top of the [Rust genai library](https://crates.io/crates/genai), which supports many top AI providers and models (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Ollama, xAI, and Cohere).

- Top new features: (see full details [CHANGELOG](CHANGELOG.md))
  - **2025-04-13 (v0.7.0) - BIG RELEASE - Windows Support (x86 & Arm), and more**
  - 2025-04-08 (v0.6.17) - Binaries available (mac/linux), pro@coder, pro@rust10x
  - 2025-03-28 - (v0.6.15) - new: `aip check-keys`
  - 2025-03-25 (v0.6.14) - agent - add `aip.flow.data_response({input?, data?, options?})` (e.g., model override by input)
  - 2025-03-20 (v0.6.12) - Now can **call agent within agents** `aip.agent.run(agent_name, {inputs?, options?})`
  - 2025-03-12 (v0.6.9) - Now supports `namespace@pack_name/some/**/*.*` for all `aip.file..`
  - 2025-03-02 (v0.6.7) - Fixes and tune-up. Pack install test and other refactoring
  - 2025-02-28 (v0.6.3) - `aip pack ..`, `aip install local...`, `ai_response.price_usd`, and more
  - 2025-02-26 (v0.6.0) - BIG UPDATE - to **AIPACK**, now with pack support (e.g., `aip run demo@craft/code`)**
  - 2025-02-22 (v0.5.11) - Huge update with parametric agents and coder (more info soon)
  - 2025-01-27 (v0.5.9) - DeepSeek distill models support for Groq and Ollama (local)
  - 2025-01-23 (v0.5.7) - `aipack run craft/text` or `aipack run craft/code` (example of new agent module support)
  - 2025-01-06 (v0.5.4) - DeepSeek `deepseek-chat` support
  - 2024-12-08 (v0.5.1) - Added support for **xAI**

## How it works

- **One Agent** == **One Markdown**
    - An `.aip` agent file is just a **Markdown file** with sections for each stage of the agent's processing.
    - See below for all the [possible stages](#multi-stage).
- `aip run demo@proof -f "./*.md"`
  - will run the installed agent file `main.aip` in the
  - pack named `proof`
  - namespace `demo`
  - agent file `main.aip`
  - Full path `~/.aipack-base/pack/installed/demo/proof/main.aip`
  - You can pass input to your agent using:
    - `-f "path/with/optional/**/glob.*" -f "README.md"` (the Lua code will receive a `{path = .., name =..}` FileMeta-like structure as input)
    -  `-i "some string" -i "another input"` (the Lua code will receive these strings as input)
    - Each input triggers one run of the agent.
- `aip run some/path/to/agent`
  - If the path ends with `.aip`, it's treated as a direct file run.
  - If there is no `.aip` extension, then:
    - `.../agent.aip` will be executed if it exists.
    - or `.../agent/main.aip` will be executed if it exists.
- **aipack** agents are simple `.aip` files that can be placed anywhere on disk.
  - e.g., `aipack run ./my-path/to/my-agent.aip ...`
- **Multi AI Provider / Models** - **aipack** uses [genai](https://crates.io/crates/genai) and therefore supports providers like OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more.
- **Lua** is used for all scripting (thanks to the great [mlua](https://crates.io/crates/mlua) crate).
- **Handlebars** is used for all prompt templating (thanks to the great Rust native [handlebars](https://crates.io/crates/handlebars) crate).

### Multi Stage

A single **aipack** file may comprise any of the following stages.

| Stage           | Language       | Description                                                                                                      |
|-----------------|----------------|------------------------------------------------------------------------------------------------------------------|
| `# Before All`  | **Lua**        | Reshape/generate inputs and add global data to the command scope (the "map" part of map/reduce).                  |
| `# Data`        | **Lua**        | Gather additional data per input and return it for the next stages.                                             |
| `# System`      | **Handlebars** | Customize the system prompt using data from `# Before All` and `# Data`.                                           |
| `# Instruction` | **Handlebars** | Customize the instruction prompt using data from `# Before All` and `# Data`.                                      |
| `# Assistant`   | **Handlebars** | Optional for special customizations, such as the "Jedi Mind Trick."                                              |
| `# Output`      | **Lua**        | Processes the `ai_response` from the LLM. If not defined, `ai_response.content` is output to the terminal.         |
| `# After All`   | **Lua**        | Called with `inputs` and `outputs` for post-processing after all input processing is complete (the "reduce" part of map/reduce). |

- `# Before All` and `# After All` act like the **map** and **reduce** steps, running before and after the main input processing loop, respectively.

[more info on stages](_init/base/pack/installed/core/doc/README.md#complete-stages-description)

## [aipack doc](_init/base/pack/installed/core/doc/README.md)

See the aipack documentation at **[core/doc/README.md](_init/base/pack/installed/core/doc/README.md)** (with the [Lua modules doc](_init/base/pack/installed/core/doc/lua.md)).

You can also run the `ask-aipack` agent.

```sh
# IMPORTANT: Make sure you have the `OPENAI_API_KEY` (or the key for your desired model) set in your environment
aip run core@ask-aipack
# The prompt file will be at `.aipack/.prompt/core@ask-aipack/ask-prompt.md`
```
