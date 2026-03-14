# 命令开发指南

本文档详细介绍了如何在 JVCS CLI 中开发新的命令。命令系统采用模块化设计，分为多个组件，每个组件有明确的职责。

## 目录结构

```
src/cmds/
├── arg/          # 命令行参数定义
├── cmd/          # 命令实现
├── collect/      # 资源收集信息
├── comp/         # 命令补全脚本
├── converter/    # 数据转换器
├── in/           # 输入数据结构
├── out/          # 输出数据结构
├── override/     # 渲染器重写
└── renderer/     # 渲染器实现
```

## 命令组件

### 1. Argument
- 实现 `clap::Parser` trait
- 命名规范：`JV{CommandName}Argument`
- 位置：`src/cmds/arg/{command_name}.rs`

示例：sum 命令的参数定义
```rust
// src/cmds/arg/sum.rs
use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVSumArgument {
    /// 要相加的数字
    pub numbers: Vec<i32>,
    
    /// 不输出结果
    #[arg(long)]
    pub no_output: bool,
}
```

### 2. Input
- 无生命周期的结构体
- 命名规范：`JV{CommandName}Input`
- 位置：`src/cmds/in/{command_name}.rs`
- 在 `prepare` 阶段由 `Argument` 转换而来

示例：sum 命令的输入结构
```rust
// src/cmds/in/sum.rs
pub struct JVSumInput {
    pub numbers: Vec<i32>,
    pub should_output: bool,
}
```

### 3. Collect
- 无生命周期的结构体
- 命名规范：`JV{CommandName}Collect`
- 位置：`src/cmds/collect/{command_name}.rs`
- 用于收集执行命令所需的本地资源信息

示例：sum 命令的资源收集
```rust
// src/cmds/collect/sum.rs
pub struct JVSumCollect {
    pub count: usize,
}
```

### 4. Output
- 实现 `serde::Serialize` trait
- 命名规范：`JV{CommandName}Output`
- 位置：`src/cmds/out/{command_name}.rs`

示例：sum 命令的输出结构
```rust
// src/cmds/out/sum.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct JVSumOutput {
    pub result: i32,
}
```

### 5. 补全脚本
- 实现命令的自动补全功能
- 命名规范：与命令同名
- 位置：`src/cmds/comp/{command_name}.rs`
- 函数签名必须为 `pub fn comp(ctx: CompletionContext) -> Option<Vec<String>>`

示例：helpdoc 命令的补全脚本
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

**补全脚本返回值说明：**
- 返回 `None`：系统会建议文件列表
- 返回 `Some(Vec::new())`：不进行任何建议
- 返回 `Some(vec!["suggestion1", "suggestion2"])`：建议具体内容

## 命令执行阶段

### 1. prepare 阶段
- 将 `Argument` 转换为稳定的 `Input` 信息
- 检测输入格式错误并提前失败
- 对输入进行格式化处理（如标志取反）

示例：sum 命令的 prepare 函数
```rust
async fn prepare(args: &JVSumArgument, ctx: &JVCommandContext) -> Result<JVSumInput, CmdPrepareError> {
    trace!("开始准备 sum 命令，参数数量: {}", args.numbers.len());
    debug!("no_output: {}, should_output: {}", args.no_output, should_output);
    
    Ok(JVSumInput {
        numbers: args.numbers.clone(),
        should_output = !args.no_output,
    })
}
```

### 2. collect 阶段
- 根据 `Argument` 的信息读取需要用到的资源
- 资源加载错误时提前失败
- 将收集的资源信息传入 `exec` 阶段

示例：sum 命令的 collect 函数
```rust
async fn collect(args: &JVSumArgument, ctx: &JVCommandContext) -> Result<JVSumCollect, CmdPrepareError> {
    trace!("收集 sum 命令资源");
    
    Ok(JVSumCollect {
        count: args.numbers.len(),
    })
}
```

### 3. exec 阶段
- 将 `prepare` 和 `collect` 阶段收集的信息绑定到核心 API
- 将结果整理为 `Output` 输出
- **必须使用** `cmd_output!(JVSomeOutput => output)` 语法实现输出

示例：sum 命令的 exec 函数
```rust
#[exec]
async fn exec(
    input: JVSumInput,
    collect: JVSumCollect,
) -> Result<(Box<dyn std::any::Any + Send + 'static>, TypeId), CmdExecuteError> {
    trace!("执行 sum 命令，处理 {} 个数字", collect.count);
    
    // 计算总和
    let result = input.numbers.iter().sum();
    debug!("计算结果: {}", result);
    
    // 根据 should_output 决定输出类型
    if input.should_output {
        cmd_output!(JVSumOutput => JVSumOutput { result })
    } else {
        // 使用 JVNoneOutput 表示不输出结果
        cmd_output!(JVNoneOutput => JVNoneOutput)
    }
}
```

## 渲染器

每个 `Output` 需要对应一个渲染器，用于将输出数据渲染为用户可读的格式。

### 渲染器实现要求
- 实现异步的 `render` 函数
- 输入为对应的 `Output` 值
- 输出为 `Result<JVRenderResult, CmdRenderError>`
- **必须使用** `#[result_renderer(JV{CommandName}Renderer)]` 宏

示例：sum 命令的渲染器
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
    trace!("渲染 sum 命令结果");
    
    let mut r = JVRenderResult::default();
    r_println!(r, "Result: {}", data.result);
    Ok(r)
}
```

## 开发流程

1. **规划命令结构**
   - 确定命令名称和参数
   - 设计输入/输出数据结构

2. **创建组件文件**
   - 在相应目录创建 `.rs` 文件
   - 实现 Argument、Input、Collect、Output 结构体

3. **实现命令逻辑**
   - 在 `cmd/` 目录创建命令实现文件
   - 使用命令模板（通过 `cargo doc --no-deps` 生成文档查看完整模板）
   - 实现 `prepare`、`collect`、`exec` 函数

4. **实现渲染器**
   - 在 `renderer/` 目录创建渲染器文件
   - 使用 `#[result_renderer]` 宏

5. **实现补全脚本 (可选)**
   - 在 `comp/` 目录创建补全脚本文件
   - 实现 `comp` 函数，签名必须为 `pub fn comp(ctx: CompletionContext) -> Option<Vec<String>>`

6. **测试命令**
   - 使用 `cargo build` 检查编译错误
   - 运行命令测试功能

## 命名规范

| 组件类型 | 命名规范 | 示例 |
|---------|---------|------|
| 命令 | `JV{CommandName}Command` | `JVSumCommand` |
| 参数 | `JV{CommandName}Argument` | `JVSumArgument` |
| 输入 | `JV{CommandName}Input` | `JVSumInput` |
| 收集 | `JV{CommandName}Collect` | `JVSumCollect` |
| 输出 | `JV{CommandName}Output` | `JVSumOutput` |
| 渲染器 | `JV{CommandName}Renderer` | `JVSumRenderer` |

## 日志输出

在命令开发中，可以使用 `log` 进行调试：

- `trace!("消息")` - 最详细的调试信息
- `debug!("消息")` - 调试信息
- `info!("消息")` - 一般信息
- `warn!("消息")` - 警告信息
- `error!("消息")` - 错误信息

## 最佳实践

1. **错误处理**
   - 在 `prepare` 阶段验证输入
   - 在 `collect` 阶段检查资源可用性

2. **输入格式化**
   - 在 `prepare` 阶段对用户输入进行标准化
   - 确保 `Input` 结构是干净、稳定的

3. **资源管理**
   - 在 `collect` 阶段获取所有需要的资源
   - 避免在 `exec` 阶段进行文件系统操作

4. **输出设计**
   - 输出结构应包含足够的信息供渲染器使用
   - 考虑不同输出格式的需求

5. **补全脚本**
   - 为常用参数提供智能补全建议
   - 根据上下文动态生成补全选项
   - 合理使用返回值控制补全行为

## 示例命令参考

查看现有命令实现以获取更多灵感：
- `helpdoc` 命令：`src/cmds/cmd/helpdoc.rs`
- `sheetdump` 命令：`src/cmds/cmd/sheetdump.rs`
- `workspace` 命令：`src/cmds/cmd/workspace.rs`

## 调试与测试

1. **生成文档查看模板**
   ```bash
   cargo doc --no-deps
   ```
   文档生成在 `.temp/target/doc/` 目录，查看 `macro.command_template.html` 获取完整命令模板。

2. **运行命令测试**
   ```bash
   # 构建并部署
   ./scripts/dev/dev_deploy.sh
   # 或 Windows
   .\scripts\dev\dev_deploy.ps1
   
   jvn sum 1 2
   ```
