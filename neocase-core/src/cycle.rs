use std::{
    hash::{Hash, Hasher},
    sync::Mutex,
};

use anyhow::Context;
use convert_case::Case;
use nvim_oxi::{Object, api::opts::OptionOpts};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHasher};

use crate::{opts::CycleVec, ring::NonemptyRing, switch::switch_to_case, visual::VisualPos};

struct LastCycle {
    hash: u64,
    cursor_start_pos: VisualPos,
    case: Case<'static>,
}

static CYCLE_CACHE: Mutex<FxHashMap<String, LastCycle>> =
    Mutex::new(FxHashMap::with_hasher(FxBuildHasher));

pub(crate) fn cycle(opts: Option<Object>) -> anyhow::Result<()> {
    let cv = opts.map(|o| CycleVec::from_obj(Some(o))).transpose()?;
    let mut win = nvim_oxi::api::get_current_win();
    let mut buf = win
        .get_buf()
        .context("failed to get buffer for current window")?;
    let opts = OptionOpts::builder().buf(buf.clone()).build();
    let ft: String = nvim_oxi::api::get_option_value("filetype", &opts)
        .context("failed to get buffer filetype")?;
    let cursor_pos = win.get_cursor().context("failed to get cursor_pos")?;

    let mut cache = CYCLE_CACHE.lock().unwrap();
    let cases = if let Some(cv) = cv {
        cv.inner
    } else if let Some(global) = crate::opts::get_global()
        .and_then(|o| o.cycle.ft_cycles.get(&ft).or(o.cycle.global.as_ref()))
    {
        global.clone()
    } else {
        let default_selection = CycleVec::default();
        default_selection.inner
    };
    let mut use_cases = next_with_cache(&ft, cases, &mut cache)?;
    let (new_pos, case) = switch_to_case(
        &mut win,
        &mut buf,
        &mut use_cases,
        cursor_pos.0,
        cursor_pos.1,
        true,
    )?;
    let cases = use_cases.into_inner();
    let lc = cache.entry(ft).or_insert_with(|| {
        let mut case_hash = FxHasher::default();
        cases.hash(&mut case_hash);
        let hash = case_hash.finish();
        LastCycle {
            hash,
            cursor_start_pos: new_pos,
            case,
        }
    });
    lc.cursor_start_pos = new_pos;

    Ok(())
}

fn next_with_cache(
    ft: &str,
    cases: Vec<Case<'static>>,
    cache: &mut FxHashMap<String, LastCycle>,
) -> anyhow::Result<NonemptyRing<Case<'static>>> {
    let mut case_hash = FxHasher::default();
    cases.hash(&mut case_hash);
    let hash = case_hash.finish();
    let Some(vp) = VisualPos::get()? else {
        return NonemptyRing::new(cases);
    };

    let Some(lc) = cache.get_mut(ft) else {
        return NonemptyRing::new(cases);
    };
    if lc.cursor_start_pos == vp && lc.hash == hash {
        let mut ring = NonemptyRing::new(cases)?;
        ring.advance_to_after(&lc.case)?;
        lc.case = *ring.get();
        Ok(ring)
    } else {
        lc.cursor_start_pos = vp;
        lc.hash = hash;
        lc.case = cases[0];
        NonemptyRing::new(cases)
    }
}
