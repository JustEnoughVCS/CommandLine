use cli_utils::display::markdown::Markdown;
use render_system_macros::result_renderer;
use rust_i18n::t;

use crate::{
    cmds::out::version::JVVersionOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

#[result_renderer(JVVersionRenderer)]
pub async fn render(data: &JVVersionOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();

    if data.show_banner {
        draw_banner(&mut r, data)
    } else {
        draw_version(&mut r, data);
    }

    if data.show_compile_info {
        draw_compile_infos(&mut r, data);
    }

    Ok(r)
}

fn draw_banner(r: &mut JVRenderResult, data: &JVVersionOutput) {
    let banner_str = t!(
        "banner",
        banner_line_1 = t!("version.banner_title_line").trim(),
        banner_line_2 = t!(
            "version.banner_cmd_version",
            cli_version = data.compile_info.cli_version,
            build_time = data.compile_info.date
        )
        .trim(),
        banner_line_3 = t!(
            "version.banner_core_version",
            core_version = data.compile_info_core.vcs_version
        )
        .trim()
    );
    let trimmed_banner_str = banner_str
        .trim_start_matches("_banner_begin")
        .trim_matches('\n');
    r_println!(r, "{}", trimmed_banner_str.to_string().markdown())
}

fn draw_version(r: &mut JVRenderResult, data: &JVVersionOutput) {
    if data.show_compile_info {
        r_println!(
            r,
            "{}",
            t!(
                "version.no_banner_output_with_compile_info",
                version = data.compile_info.cli_version
            )
            .trim()
        )
    } else {
        r_println!(
            r,
            "{}",
            t!(
                "version.no_banner_output",
                version = data.compile_info.cli_version
            )
            .trim()
        )
    }
}

fn draw_compile_infos(r: &mut JVRenderResult, data: &JVVersionOutput) {
    r_println!(
        r,
        "\n{}",
        t!(
            "version.compile_info.info",
            build_time = data.compile_info.date,
            target = data.compile_info.target,
            platform = data.compile_info.platform,
            toolchain = data.compile_info.toolchain,
            core_branch = data.compile_info_core.build_branch,
            cli_branch = data.compile_info.build_branch,
            core_commit = &data.compile_info_core.build_commit[..7],
            cli_commit = &data.compile_info.build_commit[..7]
        )
        .to_string()
        .markdown()
    );
}
