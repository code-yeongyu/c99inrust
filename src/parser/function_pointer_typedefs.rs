use super::pointer_referent_for_depth;

pub(super) fn function_pointer_typedef_declaration_referent(
    typedefs: &[(String, String)],
    base_referent: Option<&str>,
    pointer_depth: usize,
) -> Option<String> {
    let function_referent = function_pointer_typedef_referent(typedefs, base_referent?)?;
    if pointer_depth == 0 {
        return Some(function_referent);
    }
    pointer_referent_for_depth(pointer_depth, Some(&function_referent))
}

pub(super) fn function_pointer_typedef_referent(
    typedefs: &[(String, String)],
    name: &str,
) -> Option<String> {
    typedefs
        .iter()
        .find(|(typedef_name, _referent)| typedef_name == name)
        .map(|(_typedef_name, referent)| referent.clone())
}
