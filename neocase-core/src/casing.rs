use anyhow::{Context, Result, bail};
use convert_case::Case;

pub(crate) fn prompt_case() -> Result<Option<Case<'static>>> {
    // Add a dummy-function so that it can be used for completion
    // Pretty cursed tbh
    // see `vim.fn.input`
    // also :h command-completion
    nvim_oxi::api::command(
        "\
:function! Neocase_cases_completion(A, L, P)\n\
:  return \"ada\\ncamel\\ncobol\\nflat\\nkebab\\npascal\\nscream\\nsnake\\ntrain\"\n\
:endfunction",
    )
    .context("faied to add dummy shim function")?;

    let prompt: String =
        nvim_oxi::api::call_function("input", ("Case: ", "", "custom,Neocase_cases_completion"))
            .context("failed to call vim fn")?;
    input_to_case(prompt.as_str())
}

pub(crate) fn input_to_case(input: &str) -> Result<Option<Case<'static>>> {
    let matched = match input.to_lowercase().as_str() {
        "ada" => Case::Ada,
        "camel" => Case::Camel,
        "cobol" => Case::Cobol,
        "flat" => Case::Flat,
        "kebab" => Case::Kebab,
        "pascal" => Case::Pascal,
        "scream" => Case::UpperSnake,
        "snake" => Case::Snake,
        "train" => Case::Train,
        "" => return Ok(None),
        unk => bail!("unknown case {unk}"),
    };
    Ok(Some(matched))
}
