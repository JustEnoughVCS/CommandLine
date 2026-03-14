# Command Dev Guide

This doc explains how to develop new commands for JVCS CLI. The command system is modular, with distinct components.

## Directory Structure

```
src/cmds/
├── arg/          # CLI arg definitions
├── cmd/          # Command impl
├── collect/      # Resource collection
├── comp/         # Command completion scripts
├── converter/    # Data converters
├── in/           # Input data structs
├── out/          # Output data structs
├── override/     # Renderer overrides
└── renderer/     # Renderer impls
```

## Command Components

### 1. Argument
- Impl `clap::Parser` trait
- Naming: `JV{CommandName}Argument`
- Location: `src/cmds/arg/{command_name}.rs`

Example: sum command args
```rust
// src/cmds/arg/sum.rs
use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVSumArgument {
    /// Numbers to add
    pub numbers: Vec<i32>,

    /// Don't output result
    #[arg(long)]
    pub no_output: bool,
}
```

### 2. Input
- Lifetime-free struct
- Naming: `JV{CommandName}Input`
- Location: `src/cmds/in/{command_name}.rs`
- Created from `Argument` in `prepare` phase

Example: sum command input
```rust
// src/cmds/in/sum.rs
pub struct JVSumInput {
    pub numbers: Vec<i32>,
    pub should_output: bool,
}
```

### 3. Collect
- Lifetime-free struct
- Naming: `JV{CommandName}Collect`
- Location: `src/cmds/collect/{command_name}.rs`
- Collects local resources needed for cmd execution

Example: sum command collect
```rust
// src/cmds/collect/sum.rs
pub struct JVSumCollect {
    pub count: usize,
}
```

### 4. Output
- Impl `serde::Serialize` trait
- Naming: `JV{CommandName}Output`
- Location: `src/cmds/out/{command_name}.rs`

Example: sum command output
```rust
// src/cmds/out/sum.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct JVSumOutput {
    pub result: i32,
}
```

### 5. Completion Script
- Implements command auto-completion
- Naming: Same as command name
- Location: `src/cmds/comp/{command_name}.rs`
- Function signature must be `pub fn comp(ctx: CompletionContext) -> Option<Vec<String>>`

Example: helpdoc command completion script
```rust
// src/cmds/comp/helpdoc.rs
use crate::systems::{comp::context::CompletionContext, helpdoc};

pub fn comp(ctx: CompletionContext) -> Option<Vec<String>> {
    if ctx.previous_word == "helpdoc" {
        return Some(
            helpdoc::get_helpdoc_list()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
    }
    None
}
```

**Completion Script Return Value Explanation:**
- Return `None`: System will suggest file list
- Return `Some(Vec::new())`: No suggestions
- Return `Some(vec!["suggestion1", "suggestion2"])`: Suggest specific content

## Command Execution Phases

### 1. prepare phase
- Convert `Argument` to stable `Input`
- Detect input format errors early
- Format input (e.g., flag inversion)

Example: sum command prepare
```rust
async fn prepare(args: &JVSumArgument, ctx: &JVCommandContext) -> Result<JVSumInput, CmdPrepareError> {
    trace!("Preparing sum cmd, arg count: {}", args.numbers.len());
    debug!("no_output: {}, should_output: {}", args.no_output, should_output);

    Ok(JVSumInput {
        numbers: args.numbers.clone(),
        should_output = !args.no_output,
    })
}
```

### 2. collect phase
- Read resources based on `Argument` info
- Fail early on resource load errors
- Pass collected info to `exec` phase

Example: sum command collect
```rust
async fn collect(args: &JVSumArgument, ctx: &JVCommandContext) -> Result<JVSumCollect, CmdPrepareError> {
    trace!("Collecting sum cmd resources");

    Ok(JVSumCollect {
        count: args.numbers.len(),
    })
}
```

### 3. exec phase
- Bind info from `prepare` & `collect` to core API
- Organize result as `Output`
- **Must use** `cmd_output!(JVSomeOutput => output)` syntax

Example: sum command exec
```rust
#[exec]
async fn exec(
    input: JVSumInput,
    collect: JVSumCollect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    trace!("Exec sum cmd, processing {} numbers", collect.count);

    // Calculate sum
    let result = input.numbers.iter().sum();
    debug!("Result: {}", result);

    // Decide output type based on should_output
    if input.should_output {
        cmd_output!(JVSumOutput => JVSumOutput { result })
    } else {
        // Use JVNoneOutput for no result
        cmd_output!(JVNoneOutput => JVNoneOutput)
    }
}
```

## Renderer

Each `Output` needs a renderer to format data for user display.

### Renderer Requirements
- Impl async `render` function
- Input: corresponding `Output` value
- Output: `Result<JVRenderResult, CmdRenderError>`
- **Must use** `#[result_renderer(JV{CommandName}Renderer)]` macro

Example: sum command renderer
```rust
// src/cmds/renderer/sum.rs
use render_system_macros::result_renderer;

use crate::{
    cmds::out::sum::JVSumOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVSumRenderer)]
pub async fn render(data: &JVSumOutput) -> Result<JVRenderResult, CmdRenderError> {
    trace!("Rendering sum cmd result");

    let mut r = JVRenderResult::default();
    r_println!(r, "Result: {}", data.result);
    Ok(r)
}
```

## Dev Workflow

1. **Plan Command Structure**
   - Determine cmd name & args
   - Design input/output data structs

2. **Create Component Files**
   - Create `.rs` files in respective dirs
   - Impl Argument, Input, Collect, Output structs

3. **Implement Command Logic**
   - Create cmd impl file in `cmd/` dir
   - Use cmd template (view via `cargo doc --no-deps`)
   - Impl `prepare`, `collect`, `exec` functions

4. **Implement Renderer**
   - Create renderer file in `renderer/` dir
   - Use `#[result_renderer]` macro

5. **Implement Completion Script (Optional)**
   - Create completion script file in `comp/` dir
   - Implement `comp` function with signature `pub fn comp(ctx: CompletionContext) -> Option<Vec<String>>`

6. **Test Command**
   - Use `cargo build` to check compile errors
   - Run cmd to test functionality

## Naming Conventions

| Component Type | Naming Convention | Example |
|---------|---------|------|
| Command | `JV{CommandName}Command` | `JVSumCommand` |
| Argument | `JV{CommandName}Argument` | `JVSumArgument` |
| Input | `JV{CommandName}Input` | `JVSumInput` |
| Collect | `JV{CommandName}Collect` | `JVSumCollect` |
| Output | `JV{CommandName}Output` | `JVSumOutput` |
| Renderer | `JV{CommandName}Renderer` | `JVSumRenderer` |

## Logging

Use `log` for debugging during cmd dev:

- `trace!("msg")` - Most detailed debug info
- `debug!("msg")` - Debug info
- `info!("msg")` - General info
- `warn!("msg")` - Warning
- `error!("msg")` - Error

## Best Practices

1. **Error Handling**
   - Validate input in `prepare` phase
   - Check resource availability in `collect` phase

2. **Input Formatting**
   - Standardize user input in `prepare` phase
   - Ensure `Input` struct is clean & stable

3. **Resource Management**
   - Get all needed resources in `collect` phase
   - Avoid filesystem ops in `exec` phase

4. **Output Design**
   - Output struct should have enough info for renderer
   - Consider needs of different output formats

5. **Completion Scripts**
   - Provide intelligent completion suggestions for common parameters
   - Generate completion options dynamically based on context
   - Use return values appropriately to control completion behavior

## Example Commands

Check existing cmd impls for inspiration:
- `helpdoc` cmd: `src/cmds/cmd/helpdoc.rs`
- `sheetdump` cmd: `src/cmds/cmd/sheetdump.rs`
- `workspace` cmd: `src/cmds/cmd/workspace.rs`

## Debug & Test

1. **Generate Docs for Template**
   ```bash
   cargo doc --no-deps
   ```
   Docs are in `.temp/target/doc/`, see `macro.command_template.html` for full template.

2. **Run Command Test**
   ```bash
   # Build & deploy
   ./scripts/dev/dev_deploy.sh
   # or Windows
   .\scripts\dev\dev_deploy.ps1

   jvn sum 1 2
   ```
