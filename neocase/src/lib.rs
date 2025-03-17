use nvim_oxi::{Dictionary, Function, Object};

#[nvim_oxi::plugin]
pub fn neocase() -> Dictionary {
    // lib-native parsing can panic, crashing Neovim, do the conversion manually
    let setup = Function::from_fn(neocase_core::setup);
    let switch = Function::from_fn(neocase_core::switch);
    let switch_dr: Function<Option<Object>, Option<String>> =
        Function::from_fn(neocase_core::switch_dr);

    let switch_dr_exec = Function::from_fn(neocase_core::switch_dr_exec);
    let cycle = Function::from_fn(neocase_core::cycle);
    let entries: [(&str, Object); 5] = [
        ("setup", setup.into()),
        ("switch", switch.into()),
        ("switch_dr", switch_dr.into()),
        ("switch_dr_exec", switch_dr_exec.into()),
        ("cycle", cycle.into()),
    ];
    Dictionary::from_iter(entries)
}
