use std::fmt::Write;

use crate::visual::VisualPos;
use anyhow::{Context, Result};
use nvim_oxi::api::{Buffer, opts::GetTextOpts};

pub(crate) fn buf_get_text_at(buf: &Buffer, pos: &VisualPos) -> Result<String> {
    let opts = GetTextOpts::builder().build();
    let (start, end) = pos.column_range();
    let text = buf
        .get_text(pos.line_range(), start, end, &opts)
        .with_context(|| format!("failed to get text at pos: {pos:?}"))?;
    let mut s = String::new();
    for line in text {
        let _ = s.write_fmt(format_args!("{line}\n"));
    }
    // Todo: Not getting the nl on the last line, may be problematic
    s.pop();
    Ok(s)
}
