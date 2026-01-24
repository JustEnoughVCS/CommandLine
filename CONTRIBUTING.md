# Contribution Guide

Welcome to contributing to the JustEnoughVCS command-line tool! This guide is designed to help you quickly get started with the development process and understand the project's code standards. Please read the following content to ensure your contributions can be accepted smoothly.



## Local Development

### Environment Setup
1. Clone the core library ([VersionControl](https://github.com/JustEnoughVCS/VersionControl)) and your forked command-line library, arranging them in the following directory structure:

```
├─ CommandLine
│      Cargo.lock
│      Cargo.toml
└─ VersionControl
       Cargo.lock
       Cargo.toml
```

2. In the core library directory, execute `setup.sh` (Linux/macOS) or `setup.ps1` (Windows) based on your operating system.

```bash
cd VersionControl
./setup.sh
# or
.\setup.ps1
```



### Development Workflow

1.  Create a new feature branch from the `dev` branch, using the naming format `feat/xxxx`.
2.  It is recommended to add the original repository as a remote upstream to pull updates regularly:
```bash
git remote add upstream https://github.com/JustEnoughVCS/CommandLine
git pull upstream dev
```



### Building and Testing

Use `scripts/dev/dev_deploy.sh` (or `.ps1`) for test builds. The build artifacts are located in the `.temp/deploy/` directory.

- **Windows**: Add `.temp/deploy/jv_cli.ps1` to your PowerShell `$PROFILE`.

```
# ...
	
. C:\...\JustEnoughVCS\CommandLine\.temp\deploy\jv_cli.ps1
	
# ...
```
<center>C:\Users\YourName\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1</center>

- **Linux/macOS**: Add a `source` command in `.zshrc` or `.bashrc` pointing to `.temp/deploy/jv_cli.sh`.

```
# ...

source ~/.../JustEnoughVCS/CommandLine/.temp/deploy/jv_cli.sh

# ...
```
<center>/home/your_name/.bashrc | /home/your_name/.zshrc</center>

> [!TIP]
>
> For more convenient debugging,
>
> you can run `scripts/make_lnk.ps1` or `scripts/make_lnk.sh` to create shortcuts or symbolic links.
>
> *Related files are ignored by `.gitignore`*



### Submitting and Merging
-   Before pushing code, be sure to execute `scripts/dev/deploy.sh` for a formal local deployment to check for potential issues.
-   When creating a Pull Request (PR), please set the target branch to the command-line repository's `github_action/deploy`. **PRs submitted to the `main` or `dev` branches will not be processed.**



### Important Notes
-   **Rust Version**: It is recommended to use `rustc 1.92.0 (ded5c06cf 2025-12-08) (stable)`.
-   **File Size**: **Strictly prohibit** committing binary files larger than 1MB to the repository. If necessary, please discuss it first in an [Issue](https://github.com/JustEnoughVCS/CommandLine/issues).
-   **Core Library Modifications**: If you need to modify the core library, please refer to the `CONTRIBUTE.md` document in the [VersionControl](https://github.com/JustEnoughVCS/VersionControl) repository.



## Development Standards

### Code Structure

A complete command consists of the following components, organized by module:

| Module | Path | Description |
|--------|------|-------------|
| **Command Definition** | `src/cmds/` | The main logic implementation of the command. |
| **Argument Definition** | `src/args/` | Defines command-line inputs using `clap`. |
| **Input Data** | `src/inputs/` | User input data during command execution. |
| **Collected Data** | `src/collects/` | Data collected locally during command execution. |
| **Output Data** | `src/outputs/` | The command's output data. |
| **Renderer** | `src/renderers/` | The default presentation method for data. |



### Naming Conventions

- **File Naming**: Follow the format of `src/cmds/status.rs`, i.e., use the command name as the filename.
- **Multi-level Subcommands**: In the `cmds` directory, use the `sub_subsub.rs` format for filenames (e.g., `sheet_drop.rs`).
- **Struct Naming**:
				- Command Struct: `JV{Subcommand}{Subsubcommand}Command` (e.g., `JVSheetDropCommand`).
				- Other component structs follow the same pattern:
								- `JV{XXX}Argument`
								- `JV{XXX}Input`
								- `JV{XXX}Output`
								- `JV{XXX}Collect`
								- `JV{XXX}Renderer`



### Other Development Conventions
- **Utility Functions**: Reusable functionality should be placed in the `src/utils/` directory (e.g., `src/utils/feat.rs`). Test code should be written directly within the corresponding feature file.
- **Special Files**: `.rs` files starting with an underscore `_` are excluded by the `.gitignore` rule and will not be tracked by Git.
- **File Movement**: If you need to move a file, be sure to use the `git mv` command or ensure the file is already tracked by Git. The commit message should explain the reason for the move.
- **Frontend/Backend Responsibilities**: The frontend (command-line interface) should remain lightweight, primarily responsible for data collection and presentation. Any operation that needs to modify workspace data must call the interfaces provided by the core library.



### Regarding Existing Code
Please note that the above standards were established after the project stabilized. Existing code may contain parts that do not yet follow these rules. If you discover such cases, we welcome and appreciate your submissions to correct them.



# Finally
Thank you for reading this far! We look forward to your contributions and welcome you to discuss with us in the project's discussion area at any time.

Once again, thank you sincerely for your support of JustEnoughVCS!
