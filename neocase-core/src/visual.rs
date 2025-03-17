use anyhow::{Context, Result};
use nvim_oxi::{
    Array,
    api::{Window, types::Mode},
    conversion::FromObject,
};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct VisualPos {
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
}

impl VisualPos {
    pub(crate) fn create_from_inline_word(
        start_row: usize,
        start_col: usize,
        word_length: usize,
    ) -> Self {
        Self {
            start_row,
            start_col,
            end_row: start_row,
            end_col: start_col + word_length,
        }
    }
    pub(crate) fn reverse(&mut self) {
        core::mem::swap(&mut self.start_row, &mut self.end_row);
        core::mem::swap(&mut self.start_col, &mut self.end_col);
    }
    pub(crate) fn is_reversed(&self) -> bool {
        match self.start_row().cmp(&self.end_row()) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => self.start_col() > self.end_col(),
            std::cmp::Ordering::Greater => true,
        }
    }

    /// Zero-indexed start col
    #[inline]
    pub(crate) fn start_col(&self) -> usize {
        self.start_col.saturating_sub(1)
    }

    #[inline]
    pub(crate) fn start_row(&self) -> usize {
        self.start_row
    }

    #[inline]
    pub(crate) fn end_col(&self) -> usize {
        self.end_col
    }

    #[inline]
    pub(crate) fn end_row(&self) -> usize {
        self.end_row
    }

    pub(crate) fn line_range(&self) -> std::ops::Range<usize> {
        let (start, end) = if self.start_row() <= self.end_row() {
            (self.start_row(), self.end_row())
        } else {
            (self.end_row(), self.start_row())
        };
        start.saturating_sub(1)..end.saturating_sub(1)
    }

    pub(crate) fn column_range(&self) -> (usize, usize) {
        if self.start_col() <= self.end_col() {
            (self.start_col, self.end_col())
        } else {
            (
                self.end_col().saturating_sub(1),
                self.start_col().saturating_add(1),
            )
        }
    }

    pub(crate) fn set_end_col(&mut self, next: usize) {
        self.end_col = next;
    }

    pub(crate) fn get() -> Result<Option<Self>> {
        let mode = nvim_oxi::api::get_mode()
            .context("failed to get mode")?
            .mode;
        if !matches!(mode, Mode::Visual | Mode::VisualSelect) {
            return Ok(None);
        }
        let start: Array = nvim_oxi::api::call_function("getpos", ("v",))
            .context("failed to get visual start position")?;
        let start_row: i64 =
            FromObject::from_object(start.get(1).context("failed to get start row")?.clone())
                .context("start row was not an integer")?;
        let start_col: i64 =
            FromObject::from_object(start.get(2).context("failed to get start col")?.clone())
                .context("start col was not an integer")?;
        let end: Array = nvim_oxi::api::call_function("getpos", (".",))
            .context("failed to get visual end position")?;
        let end_row: i64 =
            FromObject::from_object(end.get(1).context("failed to get end row")?.clone())
                .context("end row was not an integer")?;
        let end_col: i64 =
            FromObject::from_object(end.get(2).context("failed to get end col")?.clone())
                .context("end col was not an integer")?;
        Ok(Some(Self {
            start_row: start_row
                .try_into()
                .context("failed to convert i64 start row to usize")?,
            start_col: (start_col - 1)
                .try_into()
                .context("failed to convert i64 start col to usize")?,
            end_row: end_row
                .try_into()
                .context("failed to convert i64 end row to usize")?,
            end_col: end_col
                .try_into()
                .context("failed to convert i64 end col to usize")?,
        }))
    }

    /// Will un-reverse the selected area, may be confusing but it's the only way to do it afaik
    pub(crate) fn set_visual_pos(&mut self, win: &mut Window) -> Result<()> {
        if self.is_reversed() {
            self.reverse();
        }
        // Make sure it's in normal mode
        nvim_oxi::api::command("normal! :noh").context("failed to start insert")?;
        win.set_cursor(self.start_row(), self.start_col())
            .context("failed to set cursor")?;
        let _: () = nvim_oxi::api::command("normal! v").context("failed to enter visual mode")?;
        win.set_cursor(self.end_row(), self.end_col())
            .context("failed to set cursor")?;
        Ok(())
    }
}
