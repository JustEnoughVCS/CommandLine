use cli_utils::{
    display::{SimpleTable, md},
    env::auto_update_outdate,
};
use render_system_macros::result_renderer;
use rust_i18n::t;

use crate::cmds::out::status::JVStatusWrongModifyReason;
use crate::{
    cmds::out::status::JVStatusOutput,
    r_println,
    systems::{cmd::errors::CmdRenderError, render::renderer::JVRenderResult},
};

enum Mode {
    StructuralChangesMode,
    ContentChangesMode,
    Clean,
}

#[result_renderer(JVStatusRenderer)]
pub async fn render(data: &JVStatusOutput) -> Result<JVRenderResult, CmdRenderError> {
    let mut r = JVRenderResult::default();

    // Render Header
    render_header(&mut r, data);

    // Render Info and Mode
    render_info_and_mode(&mut r, data);

    // Render Hint
    render_hint(&mut r, data);

    Ok(r)
}

fn render_header(r: &mut JVRenderResult, data: &JVStatusOutput) {
    let account = &data.current_account;
    let sheet = &data.current_sheet;
    r_println!(
        r,
        "{}",
        md(t!("status.header", account = account, sheet = sheet))
    );
}

fn render_info_and_mode(r: &mut JVRenderResult, data: &JVStatusOutput) {
    let mut info_erased = String::default();
    let mut info_moved = String::default();
    let mut info_lost = String::default();
    let mut info_created = String::default();
    let mut info_modified = String::default();

    // Collect erased items
    if !data.analyzed_result.erased.is_empty() {
        info_erased.push_str(format!("{}\n", md(t!("status.info_display.erased.header"))).as_str());
        for erased in data.analyzed_result.erased.iter() {
            info_erased.push_str(
                format!(
                    "{}\n",
                    md(t!(
                        "status.info_display.erased.item",
                        item = erased.display()
                    ))
                )
                .as_str(),
            );
        }
    }

    // Collect moved items
    if !data.analyzed_result.moved.is_empty() {
        let mut table = SimpleTable::new(vec![
            format!("{}", md(t!("status.info_display.moved.header"))),
            "".to_string(),
        ]);
        for (_, (from, to)) in data.analyzed_result.moved.iter() {
            table.push_item(vec![
                format!(
                    "{}",
                    md(t!("status.info_display.moved.left", left = from.display()))
                ),
                format!(
                    "{}",
                    md(t!("status.info_display.moved.right", right = to.display()))
                ),
            ]);
        }
        info_moved.push_str(table.to_string().as_str());
    }

    // Collect lost items
    if !data.analyzed_result.lost.is_empty() {
        info_lost.push_str(format!("{}\n", md(t!("status.info_display.lost.header"))).as_str());
        for lost in data.analyzed_result.lost.iter() {
            info_lost.push_str(
                format!(
                    "{}\n",
                    md(t!("status.info_display.lost.item", item = lost.display()))
                )
                .as_str(),
            );
        }
    }

    // Collect created items
    if !data.analyzed_result.created.is_empty() {
        info_created
            .push_str(format!("{}\n", md(t!("status.info_display.created.header"))).as_str());
        for created in data.analyzed_result.created.iter() {
            info_created.push_str(
                format!(
                    "{}\n",
                    md(t!(
                        "status.info_display.created.item",
                        item = created.display()
                    ))
                )
                .as_str(),
            );
        }
    }

    // Collect modified items
    if !data.analyzed_result.modified.is_empty() {
        info_modified
            .push_str(format!("{}\n", md(t!("status.info_display.modified.header"))).as_str());
        for modified in data.analyzed_result.modified.iter() {
            if let Some(reason) = data.wrong_modified_items.get(modified) {
                let reason_str = match reason {
                    JVStatusWrongModifyReason::BaseVersionMismatch {
                        base_version,
                        latest_version,
                    } => md(t!(
                        "status.info_display.modified.reason.base_version_mismatch",
                        base_version = base_version,
                        latest_version = latest_version
                    )),
                    JVStatusWrongModifyReason::ModifiedButNotHeld { holder } => md(t!(
                        "status.info_display.modified.reason.modified_but_not_held",
                        holder = holder
                    )),
                    JVStatusWrongModifyReason::NoHolder => {
                        md(t!("status.info_display.modified.reason.no_holder"))
                    }
                };
                info_modified.push_str(
                    format!(
                        "{}\n",
                        md(t!(
                            "status.info_display.modified.item_wrong",
                            item = modified.display(),
                            reason = reason_str
                        ))
                    )
                    .as_str(),
                );
                continue;
            }
            info_modified.push_str(
                format!(
                    "{}\n",
                    md(t!(
                        "status.info_display.modified.item",
                        item = modified.display()
                    ))
                )
                .as_str(),
            );
        }
    }

    let structural_info = vec![info_erased, info_moved, info_lost].join("\n");
    let content_info = vec![info_created, info_modified].join("\n");

    let mode = get_mode(data);
    match mode {
        Mode::StructuralChangesMode => {
            r_println!(
                r,
                "{}",
                md(t!("status.current_mode.structural", info = structural_info))
            );
        }
        Mode::ContentChangesMode => {
            r_println!(
                r,
                "{}",
                md(t!("status.current_mode.content", info = content_info))
            );
        }
        Mode::Clean => r_println!(r, "{}", md(t!("status.current_mode.clean"))),
    }
}

fn render_hint(r: &mut JVRenderResult, data: &JVStatusOutput) {
    // Outdate Hint
    let update_time = &data.update_time;
    let now_time = &data.now_time;
    let duration_minutes: i64 = (now_time
        .duration_since(*update_time)
        .unwrap_or_default()
        .as_secs()
        / 60) as i64;
    let outdate_minutes = auto_update_outdate();

    // Outdated
    if duration_minutes > outdate_minutes {
        let hours = duration_minutes / 60;
        let minutes = duration_minutes % 60;
        let seconds = (now_time
            .duration_since(*update_time)
            .unwrap_or_default()
            .as_secs()
            % 60) as i64;

        r_println!(
            r,
            "{}",
            md(t!(
                "status.hints.outdate",
                h = hours,
                m = minutes,
                s = seconds
            ))
        );
    }

    let in_ref_sheet = &data.in_ref_sheet;
    let is_host_mode = &data.is_host_mode;

    // Readonly
    if *in_ref_sheet && !is_host_mode {
        r_println!(r, "{}", md(t!("status.hints.readonly")));
    }

    // Host
    if *is_host_mode {
        r_println!(r, "{}", md(t!("status.hints.host")));
    }
}

fn get_mode(data: &JVStatusOutput) -> Mode {
    let analyzed = &data.analyzed_result;

    // If there are any lost, moved, or erased items, use structural changes mode
    if !analyzed.moved.is_empty() || !analyzed.lost.is_empty() || !analyzed.erased.is_empty() {
        Mode::StructuralChangesMode
    }
    // Otherwise, if there are any created or modified items, use content changes mode
    else if !analyzed.created.is_empty() || !analyzed.modified.is_empty() {
        Mode::ContentChangesMode
    }
    // Otherwise, it's clean
    else {
        Mode::Clean
    }
}
