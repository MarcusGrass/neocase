use std::sync::OnceLock;

use anyhow::{Context, bail};
use convert_case::Case;
use nvim_oxi::{Array, Dictionary, Object, ObjectKind, conversion::FromObject};
use rustc_hash::FxHashMap;

use crate::casing::input_to_case;

static GLOBAL_OPTS: OnceLock<NeocaseOpts> = OnceLock::new();

#[inline]
pub(crate) fn get_global() -> Option<&'static NeocaseOpts> {
    GLOBAL_OPTS.get()
}

pub(crate) fn setup(opts: Option<Object>) -> anyhow::Result<()> {
    let global = NeocaseOpts::from_obj(opts)?;
    GLOBAL_OPTS
        .set(global)
        .map_err(|_e| anyhow::anyhow!("Neocase global opts already set"))?;
    Ok(())
}

#[derive(Default)]
pub(crate) struct NeocaseOpts {
    pub(crate) cycle: CycleOpts,
    pub(crate) switch: Option<SwitchOpts>,
}

impl NeocaseOpts {
    pub(crate) fn from_obj(object: Option<Object>) -> anyhow::Result<Self> {
        let Some(opts) = object else {
            return Ok(Self::default());
        };
        let dict =
            Dictionary::from_object(opts).context("expected neocase opts to be a dictionary")?;
        let mut cycle = None;
        let mut switch = None;
        for (k, v) in dict {
            match k.to_str().context("neocase opts key was not utf8")? {
                "cycle" => {
                    if cycle.is_some() {
                        bail!("cycle supplied twice in setup");
                    }
                    cycle = Some(CycleOpts::from_obj(Some(v))?);
                }
                "switch" => {
                    if switch.is_some() {
                        bail!("switch supplied twice in setup");
                    }
                    switch = Some(SwitchOpts::from_obj(Some(v))?);
                }
                unk => bail!("unrecognized field {unk}"),
            }
        }
        Ok(Self {
            cycle: cycle.unwrap_or_default(),
            switch,
        })
    }
}

#[derive(Default, PartialEq, Eq)]
pub(crate) struct CycleOpts {
    pub(crate) global: Option<Vec<Case<'static>>>,
    pub(crate) ft_cycles: FxHashMap<String, Vec<Case<'static>>>,
}

impl CycleOpts {
    pub(crate) fn from_obj(object: Option<Object>) -> anyhow::Result<Self> {
        let Some(opts) = object else {
            return Ok(Self::default());
        };
        let dict =
            Dictionary::from_object(opts).context("expected cycle opts to be a dictionary")?;
        let mut ft_cycles = None;
        let mut global = None;
        for (k, v) in dict {
            match k.to_str().context("cycle opt not utf8")? {
                "global" => {
                    if global.is_some() {
                        bail!("cycle global already set");
                    }
                    let cv = CycleVec::from_obj(Some(v))?;
                    global = Some(cv.inner);
                }
                "ft" => {
                    let inner =
                        Dictionary::from_object(v).context("expected cycle ft to be a table")?;
                    if ft_cycles.is_some() {
                        bail!("cycle ft already set");
                    }
                    let mut ft_cycle_map = FxHashMap::default();
                    for (ft, v) in inner {
                        let ft = ft.to_string();
                        let cases = CycleVec::from_obj(Some(v))?;
                        ft_cycle_map.insert(ft, cases.inner);
                    }
                    ft_cycles = Some(ft_cycle_map);
                }
                unk => {
                    bail!("unknown cycle opt {unk}")
                }
            }
        }

        Ok(Self {
            ft_cycles: ft_cycles.unwrap_or_default(),
            global,
        })
    }
}

#[derive(Debug, Hash)]
pub(crate) struct CycleVec {
    pub(crate) inner: Vec<Case<'static>>,
}

impl Default for CycleVec {
    fn default() -> Self {
        Self {
            inner: vec![
                Case::Snake,
                Case::Pascal,
                Case::Kebab,
                Case::UpperSnake,
                Case::Camel,
                Case::Ada,
                Case::Cobol,
                Case::Train,
            ],
        }
    }
}

impl CycleVec {
    pub(crate) fn from_obj(opts: Option<Object>) -> anyhow::Result<Self> {
        let Some(opts) = opts else {
            return Ok(Self::default());
        };
        if opts.kind() != ObjectKind::Array {
            bail!("an array is the only valid cycle opts parameter");
        }
        let arr = Array::from_object(opts)
            .context("failed to cast lua object into an array after check")?;
        let mut cases = Vec::with_capacity(arr.len());
        for inner in arr {
            let raw_case = String::from_object(inner)
                .context("expected a string case inside cycle opts array, found something else")?;
            let case = input_to_case(&raw_case)
                .context("failed to parse case")?
                .context("found empty case")?;
            cases.push(case);
        }
        Ok(Self { inner: cases })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SwitchOpts {
    #[default]
    Prompt,
    RepeatLastOrFail,
    RepeatLastOrPrompt,
    Case(Case<'static>),
}

impl SwitchOpts {
    pub fn from_obj(object: Option<Object>) -> anyhow::Result<Self> {
        let Some(obj) = object else {
            return Ok(Self::default());
        };
        match obj.kind() {
            nvim_oxi::ObjectKind::String => {
                let opt = String::from_object(obj)
                    .context("bug, failed to convert checked object to string")?;
                match opt.as_str() {
                    "prompt" => Ok(Self::Prompt),
                    "repeat-or-fail" => Ok(Self::RepeatLastOrFail),
                    "repeat-or-prompt" => Ok(Self::RepeatLastOrPrompt),
                    unk => bail!("unrecognized option {unk}"),
                }
            }
            nvim_oxi::ObjectKind::Dictionary => {
                let dict = Dictionary::from_object(obj)
                    .context("bug, failed to convert checked object to dict")?;
                let mut use_case = None;
                for (k, v) in dict {
                    match k.to_str().context("dict contained non-utf8")? {
                        "case" => {
                            let raw_case_input = String::from_object(v)
                                .context("failed to convert opts case into a string")?;
                            let case =
                                input_to_case(&raw_case_input)?.context("empty case specified")?;
                            if use_case.is_some() {
                                bail!("case option specified more than once");
                            }
                            use_case = Some(case);
                        }
                        unk => {
                            bail!("unrecognized option {unk}");
                        }
                    }
                }
                if let Some(case) = use_case {
                    Ok(Self::Case(case))
                } else {
                    Ok(Self::default())
                }
            }
            k => bail!("unexpected option object kind = {k:?}"),
        }
    }
}
