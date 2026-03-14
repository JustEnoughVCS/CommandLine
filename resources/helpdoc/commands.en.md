> Welcome to the `JustEnoughVCS` CLI!

## Basic Usage
jvn <SUBCOMMAND> <ARGUMENT: ?> <FLAG: ?>

## Global Flags
Global flags for command debugging.

### Language
**Flag**: `--lang <LANGUAGE>`
Set program language.
Options: en / zh-CN

### Confirm
**Flag**: `--confirm` or `-C`
Skip confirmation prompts.

### Help
**Flag**: `--help` or `-h`
Show command help.
> Or use `jvn helpdoc <DOCUMENT>` 
> for full docs.

### Version
**Flag**: `--version` or `-v`
Redirects the current command to the `version` command
to display version information.
> For usage of the version command,
> see `commands/version`

### Renderer Override
**Flag**: `--renderer <RENDERER>`
Override output format.
Options: json / json-pretty / 
__         ron  / ron-pretty  / 
__         yaml / toml

### No Error Output
**Flag**: `--no-error-logs`
Suppress error output.

### No Progress Bar
**Flag**: `--no-progress`
Disable progress bar.

### Quiet Output
**Flag**: `--quiet` or `-q`
Suppress all output for scripts.

### Log Output
**Flag**: `--verbose` or `-V`
Enable full output for debugging.
