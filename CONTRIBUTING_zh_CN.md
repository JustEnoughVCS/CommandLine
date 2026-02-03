# 贡献指南

欢迎您愿意为 JustEnoughVCS 命令行工具贡献代码！本指南旨在帮助您快速上手开发流程，并了解项目的代码规范。请先阅读以下内容，以确保您的贡献能够顺利被接受。



## 本地开发流程

### 环境准备
1. 克隆核心库（[VersionControl](https://github.com/JustEnoughVCS/VersionControl)）以及您分叉的命令行库，按照如下文件格式放置

```
├─ CommandLine
│      Cargo.lock
│      Cargo.toml
└─ VersionControl
       Cargo.lock
       Cargo.toml
```

2. 在核心库目录下，根据您的操作系统执行 `setup.sh` (Linux/macOS) 或 `setup.ps1` (Windows)。


```bash
cd VersionControl
./setup.sh
# 或
.\setup.ps1
```



### 开发流程

1.  从 `dev` 分支创建新的功能分支，命名格式为 `feat/xxxx`。
2.  建议将原始仓库添加为远程上游仓库，以便定期拉取更新：
    ```bash
    git remote add upstream https://github.com/JustEnoughVCS/CommandLine
    git pull upstream dev
    ```



### 构建与测试

使用 `scripts/dev/dev_deploy.sh` (或 `.ps1`) 进行测试构建。构建产物位于 `.temp/deploy/` 目录。

- **Windows**: 将 `.temp/deploy/jv_cli.ps1` 添加到您的 PowerShell `$PROFILE` 中。

```
# ...
  
. C:\...\JustEnoughVCS\CommandLine\.temp\deploy\jv_cli.ps1
  
# ...
```
<center>C:\Users\YourName\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1</center>

- **Linux/macOS**: 在 `.zshrc` 或 `.bashrc` 中添加 `source` 命令指向 `.temp/deploy/jv_cli.sh`。

```
# ...

sources ~/.../JustEnoughVCS/CommandLine/.temp/deploy/jv_cli.sh

# ...
```
<center>/home/your_name/.bashrc | /home/your_name/.zshrc</center>

> [!TIP]
>
> 如需更方便地调试，
>
> 可运行 `scripts/make_lnk.ps1` 或 `scripts/make_lnk.sh` 来创建快捷方式或符号链接
>
> *相关文件已被 `.gitignore` 忽略*



### 提交与合并
-   在推送代码前，请务必执行 `scripts/dev/deploy.sh` 进行一次正式的本地部署，以检查潜在问题
-   创建 Pull Request (PR) 时，请将目标分支设置为命令行仓库的 `deploy/nightly`。**提交至 `main` 或 `dev` 分支的 PR 将不予处理**



### 注意事项
-   **Rust 版本**: 推荐使用 `rustc 1.92.0 (ded5c06cf 2025-12-08) (stable)`
-   **文件大小**: **严禁** 向仓库提交超过 1MB 的二进制文件，如有必要，请先在 [Issue](https://github.com/JustEnoughVCS/CommandLine/issues) 中讨论
-   **核心库修改**: 如需修改核心库，请参考 [VersionControl](https://github.com/JustEnoughVCS/VersionControl) 仓库中的 `CONTRIBUTE.md` 文档



## 开发规范

### 代码结构

一个完整的命令由以下几个部分组成，请按模块组织：

| 模块 | 路径 | 说明 |
|------|------|------|
| **命令定义** | `src/cmds/cmd/` | 命令的主逻辑实现。 |
| **参数定义** | `src/cmds/arg/` | 使用 `clap` 定义命令行输入。 |
| **输入数据** | `src/cmds/in/` | 命令运行阶段的用户输入数据。 |
| **收集数据** | `src/cmds/collect/` | 命令运行阶段从本地收集的数据。 |
| **输出数据** | `src/cmds/out/` | 命令的输出数据。 |
| **渲染器** | `src/cmds/renderer/` | 数据的默认呈现方式。 |



### 命名规范

- **文件命名**: 请遵循 `src/cmds/cmd/status.rs` 的格式，即使用命令名称作为文件名
- **多级子命令**: 在 `cmds` 目录下，使用 `sub_subsub.rs` 格式命名文件（例如：`sheet_drop.rs`）
- **结构体命名**:
    - 命令结构体: `JV{Subcommand}{Subsubcommand}Command` (例如：`JVSheetDropCommand`)
    - 其他组件结构体遵循相同模式：
        - `JV{XXX}Argument`
        - `JV{XXX}Input`
        - `JV{XXX}Output`
        - `JV{XXX}Collect`
        - `JV{XXX}Renderer`
        
        

### 其他开发约定
- **工具函数**: 可复用的功能应置于 `utils/` 目录下（例如 `utils/feat.rs`），测试代码应直接写在对应的功能文件内
- **特殊文件**: 以 `_` 下划线开头的 `.rs` 文件已被 `.gitignore` 规则排除，不会被 Git 追踪
- **文件移动**: 如需移动文件，请务必使用 `git mv` 命令或确保文件已被 Git 追踪。在提交信息中应说明移动原因
- **前后端职责**: 前端（命令行界面）应保持轻量，主要负责数据收集与展示。任何需要修改工作区数据的操作，都必须调用核心库提供的接口



### 关于现有代码
请注意，上述规范是在项目趋于稳定后制定的。现有代码中可能存在尚未遵循这些规则的部分。如果您发现了此类情况，我们非常欢迎并感谢您提交修正。



# 最后
感谢您耐心阅读至此！期待您的贡献，并欢迎随时在项目讨论区与我们交流。

再次衷心感谢您对 JustEnoughVCS 的支持！
