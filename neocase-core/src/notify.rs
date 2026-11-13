use nvim_oxi::api::opts::EchoOpts;

pub(crate) fn notify_error(msg: &str) {
    let _ = nvim_oxi::api::echo(
        [(msg, None::<&str>)],
        true,
        &EchoOpts::builder().err(true).build(),
    );
}
