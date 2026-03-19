> `JustEnoughVCS` 命令行程序环境变量设置

## 用法
环境变量可以在调用命令前设置，用法取决于您的系统：

**Linux/macOS (Bash/Zsh等):**
__  JV\_KEY=value jvn <子命令>

**Windows (命令提示符):**
__  set JV\_KEY=value && jvn <子命令>

**Windows (PowerShell):**
__  $env:JV\_KEY="value"; jvn <子命令>

## 环境变量
`jvn` 提供若干环境变量，用来控制命令行的部分行为逻辑

### 默认编辑器
**键**：JV\_TEXT\_EDITOR
**值**：[二进制程序]

例如：
__  JV\_TEXT\_EDITOR="nano"

### 帮助文档查看器
**键**：JV\_HELPDOC\_VIEWER
**值**：[启用：1] 或 [禁用：0]

例如：
__  # 关闭帮助文档查看器输出
__  JV\_HELPDOC\_VIEWER=0 jvn -h

### 语言
**键**：JV\_LANG
**值**: [语言]

例如：
__  JV\_LANG=zh-CN jvn -v

### 分页器
**键**：JV\_PAGER
**值**：[二进制程序]

例如：
__  JV\_PAGER="less"
