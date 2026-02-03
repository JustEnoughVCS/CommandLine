pub const COMMANDS_PATH: &str = "./src/cmds/cmd/";
pub const RENDERERS_PATH: &str = "./src/cmds/renderer/";

pub const COMPILE_INFO_RS_TEMPLATE: &str = "./templates/compile_info.rs.template";
pub const COMPILE_INFO_RS: &str = "./src/data/compile_info.rs";

pub const SETUP_JV_CLI_ISS_TEMPLATE: &str = "./templates/setup_jv_cli.iss.template";
pub const SETUP_JV_CLI_ISS: &str = "./scripts/setup/windows/setup_jv_cli.iss";

pub const COMMAND_LIST_TEMPLATE: &str = "./templates/_commands.rs.template";
pub const COMMAND_LIST: &str = "./src/systems/cmd/_commands.rs";

pub const OVERRIDE_RENDERER_DISPATCHER_TEMPLATE: &str =
    "./templates/_override_renderer_dispatcher.rs.template";
pub const OVERRIDE_RENDERER_DISPATCHER: &str =
    "./src/systems/render/_override_renderer_dispatcher.rs";

pub const OVERRIDE_RENDERER_ENTRY_TEMPLATE: &str =
    "./templates/_override_renderer_entry.rs.template";
pub const OVERRIDE_RENDERER_ENTRY: &str = "./src/systems/render/_override_renderer_entry.rs";

pub const SPECIFIC_RENDERER_MATCHING_TEMPLATE: &str =
    "./templates/_specific_renderer_matching.rs.template";
pub const SPECIFIC_RENDERER_MATCHING: &str = "./src/systems/render/_specific_renderer_matching.rs";

pub const REGISTRY_TOML: &str = "./.cargo/registry.toml";

pub const TEMPLATE_START: &str = "// -- TEMPLATE START --";
pub const TEMPLATE_END: &str = "// -- TEMPLATE END --";
