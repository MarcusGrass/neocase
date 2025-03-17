use std::sync::Mutex;

use anyhow::{Context, Result};
use convert_case::Case;
use nvim_oxi::api::{Buffer, Window};

use crate::{
    buf::buf_get_text_at, casing::prompt_case, opts::SwitchOpts, ring::NonemptyRing,
    visual::VisualPos,
};

static LAST_CASE: Mutex<Option<Case<'static>>> = Mutex::new(None);

pub(crate) fn switch(opts: SwitchOpts) -> Result<()> {
    let mut win = nvim_oxi::api::get_current_win();
    let mut buf = win
        .get_buf()
        .context("failed to get curreent window's buffer")?;
    let (row, col) = win.get_cursor().context("failed to get cursor")?;
    let case = match opts {
        SwitchOpts::Prompt => {
            let Some(case) = prompt_case()? else {
                return Ok(());
            };
            *LAST_CASE.lock().unwrap() = Some(case);
            case
        }
        SwitchOpts::RepeatLastOrFail => LAST_CASE
            .lock()
            .unwrap()
            .context("failed to find last case on repeat-or-fail")?,
        SwitchOpts::RepeatLastOrPrompt => {
            let mut guard = LAST_CASE.lock().unwrap();
            if let Some(g) = guard.as_ref() {
                *g
            } else {
                let Some(case) = prompt_case()? else {
                    return Ok(());
                };

                *guard = Some(case);
                case
            }
        }
        SwitchOpts::Case(case) => {
            *LAST_CASE.lock().unwrap() = Some(case);
            case
        }
    };
    let mut ring = NonemptyRing::new(vec![case])?;
    switch_to_case(&mut win, &mut buf, &mut ring, row, col, false)?;
    Ok(())
}

enum TextSwitchMode {
    Visual(VisualPos),
    Normal,
}

#[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub(crate) fn switch_to_case(
    win: &mut Window,
    buf: &mut Buffer,
    cases: &mut NonemptyRing<Case<'static>>,
    row: usize,
    col: usize,
    enforce_visual: bool,
) -> Result<(VisualPos, Case<'static>)> {
    let (text, normal) = if let Some(pos) = VisualPos::get()? {
        (buf_get_text_at(buf, &pos)?, TextSwitchMode::Visual(pos))
    } else {
        (
            nvim_oxi::api::call_function("expand", ("<cword>",))?,
            TextSwitchMode::Normal,
        )
    };
    let use_case = cases.get_and_progress();
    let mut final_case = use_case;
    let conv = convert_case::Converter::new().to_case(use_case);
    let mut out = conv.convert(&text);
    if out == text {
        for case in cases.iter_from(&use_case) {
            let conv = convert_case::Converter::new().to_case(case);
            out = conv.convert(&text);
            final_case = case;
            if out != text {
                break;
            }
        }
    }
    let target_row = row.saturating_sub(1);
    let mut lines = buf
        .get_lines(target_row..=target_row, false)
        .context("failed to get lines")?;
    let line = lines.next().context("failed to get case convert line")?;
    let target_line = line
        .to_str()
        .context("failed to find convert case word, found non-utf8 on line")?;
    let text_chars = text.chars().count();
    let word_start = if matches!(normal, TextSwitchMode::Normal) {
        col
    } else {
        col - text_chars + 1
    };
    let mut start_col = None;
    for (hit, _text) in target_line.match_indices(&text) {
        if hit == word_start {
            start_col = Some(word_start);
            break;
        }
    }
    let start_col = start_col.with_context(|| format!("failed to find word {text:?} in buffer"))?;
    buf.set_text(
        target_row..target_row,
        start_col,
        start_col + text_chars,
        [out.as_str()],
    )
    .context("failed to replace case word")?;
    let out_chars = out.chars().count();
    let delta = out_chars as isize - text_chars as isize;
    let vp = match normal {
        TextSwitchMode::Visual(mut visual_pos) => {
            if delta != 0 {
                let next_col = if visual_pos.is_reversed() {
                    visual_pos.set_visual_pos(win)?;
                    (visual_pos.end_col() as isize + delta).max(0) as usize
                } else {
                    (col as isize + delta).max(0) as usize
                };
                visual_pos.set_end_col(next_col + 1);
                win.set_cursor(row, next_col)
                    .context("failed to move cursor to the correct position")?;
            }
            visual_pos
        }
        TextSwitchMode::Normal => {
            let mut vp = VisualPos::create_from_inline_word(
                row,
                col.saturating_add(1),
                out_chars.saturating_sub(2),
            );
            if enforce_visual {
                vp.set_visual_pos(win)?;
            }
            vp
        }
    };

    Ok((vp, final_case))
}
