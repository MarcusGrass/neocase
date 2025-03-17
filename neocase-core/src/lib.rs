use std::sync::Mutex;

use nvim_oxi::{Dictionary, Object, api::opts::OptionOptsBuilder};
use opts::{SwitchOpts, get_global};
mod buf;
mod casing;
mod cycle;
mod opts;
mod ring;
mod switch;
mod visual;

pub fn setup(opts: Option<Object>) {
    if let Err(e) = opts::setup(opts) {
        let _ = nvim_oxi::api::notify(
            &format!("[neocase]: Failed to setup -> {e:#?}"),
            nvim_oxi::api::types::LogLevel::Error,
            &Dictionary::new(),
        );
    }
}

pub fn switch(opts: Option<Object>) {
    let so = if let Some(opts) = opts {
        SwitchOpts::from_obj(Some(opts))
    } else {
        Ok(get_global().and_then(|g| g.switch).unwrap_or_default())
    };
    match so {
        Ok(opts) => {
            if let Err(e) = switch::switch(opts) {
                let _ = nvim_oxi::api::notify(
                    &format!("[neocase]: Failed to switch case -> {e:#?}"),
                    nvim_oxi::api::types::LogLevel::Error,
                    &Dictionary::new(),
                );
            }
        }
        Err(e) => {
            let _ = nvim_oxi::api::notify(
                &format!("[neocase]: Invalid options -> {e:#?}"),
                nvim_oxi::api::types::LogLevel::Error,
                &Dictionary::new(),
            );
        }
    }
}

#[expect(clippy::missing_panics_doc)]
pub fn switch_dr(opts: Option<Object>) -> Option<String> {
    let so = if let Some(opts) = opts {
        SwitchOpts::from_obj(Some(opts))
    } else {
        Ok(get_global().and_then(|g| g.switch).unwrap_or_default())
    };
    match so {
        Ok(switch_opts) => {
            let mut opts = OptionOptsBuilder::default();
            opts.scope(nvim_oxi::api::opts::OptionScope::Global);
            *OPERATOR_FUNC_ARGS.lock().unwrap() = Some(switch_opts);
            let _ = nvim_oxi::api::set_option_value(
                "operatorfunc",
                "v:lua.require'neocase'.switch_dr_exec",
                &opts.build(),
            );
            return Some("g@l".to_string());
        }
        Err(e) => {
            let _ = nvim_oxi::api::notify(
                &format!("[neocase]: Invalid options -> {e:#?}"),
                nvim_oxi::api::types::LogLevel::Error,
                &Dictionary::new(),
            );
        }
    }
    None
}

static OPERATOR_FUNC_ARGS: Mutex<Option<SwitchOpts>> = Mutex::new(None);

#[expect(clippy::missing_panics_doc)]
pub fn switch_dr_exec(_opts: Option<Object>) {
    let mut guard = OPERATOR_FUNC_ARGS.lock().unwrap();
    if let Some(opts) = *guard {
        if let Err(e) = switch::switch(opts) {
            let _ = nvim_oxi::api::notify(
                &format!("[neocase]: Failed to switch case -> {e:#?}"),
                nvim_oxi::api::types::LogLevel::Error,
                &Dictionary::new(),
            );
            *guard = None;
        }
    }
}

pub fn cycle(opts: Option<Object>) {
    if let Err(e) = crate::cycle::cycle(opts) {
        let _ = nvim_oxi::api::notify(
            &format!("[neocase]: Failed to cycle case -> {e:#?}"),
            nvim_oxi::api::types::LogLevel::Error,
            &Dictionary::new(),
        );
    }
}
