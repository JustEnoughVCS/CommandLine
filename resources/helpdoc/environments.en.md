> `JustEnoughVCS` CLI Environments

## Usage
Environment variables can be set before invoking a command. 
The usage depends on your system:

**Linux/macOS (Bash/Zsh, etc.):**
__  JV\_KEY=value jvn <subcommand>

**Windows (Command Prompt):**
__  set JV\_KEY=value && jvn <subcommand>

**Windows (PowerShell):**
__  $env:JV\_KEY="value"; jvn <subcommand>

## Environment Variables
`jvn` provides several environment variables to 
__  control certain behaviors of the command line.

### Default Text Editor
**Key**: JV\_TEXT\_EDITOR
**Value**: [Program]

Example:
__  JV\_TEXT\_EDITOR="nano"

### Help Documentation Viewer
**Key**: JV\_HELPDOC\_VIEWER
**Value**: [Enable: 1] or [Disable: 0]

Example:
__  # Turn off help documentation viewer output
__  JV\_HELPDOC\_VIEWER=0 jvn -h

### Language
**Key**: JV\_LANG
**Value**: [Language]

Example:
__  JV\_LANG=en jvn -v

### Pager
**Key**: JV\_PAGER
**Value**: [Program]

Example:
__  JV\_PAGER="less"
