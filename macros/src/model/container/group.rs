use super::RawContainer;
use crate::model::attr::{Key, dispatch, parse_string, reject_value, set_once};
use syn::{LitStr, meta::ParseNestedMeta};

const GROUP_KEYS: &[Key<RawGroup>] = &[
    Key::new("id", id),
    Key::new("multiple", multiple),
    Key::new("required", required),
];

pub(crate) struct GroupDecl {
    pub(crate) id: LitStr,
    pub(crate) required: bool,
    pub(crate) multiple: bool,
}

#[derive(Default)]
struct RawGroup {
    id: Option<LitStr>,
    required: Option<()>,
    multiple: Option<()>,
}

fn id(meta: &ParseNestedMeta, raw: &mut RawGroup) -> syn::Result<()> {
    let literal = parse_string(meta, "id", "id = \"mode\"")?;
    if literal.value().is_empty() {
        return Err(syn::Error::new(literal.span(), "`id` cannot be empty"));
    }
    set_once(&mut raw.id, literal, meta)
}

fn required(meta: &ParseNestedMeta, raw: &mut RawGroup) -> syn::Result<()> {
    reject_value(meta, "required")?;
    set_once(&mut raw.required, (), meta)
}

fn multiple(meta: &ParseNestedMeta, raw: &mut RawGroup) -> syn::Result<()> {
    reject_value(meta, "multiple")?;
    set_once(&mut raw.multiple, (), meta)
}

pub(super) fn group(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    const EXAMPLE: &str = "group(id = \"mode\", required)";
    if !meta.input.peek(syn::token::Paren) {
        return Err(meta.error(format!("`group` expects a list, e.g. `{EXAMPLE}`")));
    }
    let mut group = RawGroup::default();
    meta.parse_nested_meta(|inner| dispatch(inner, "group", GROUP_KEYS, &mut group))?;
    let Some(id) = group.id else {
        return Err(meta.error(format!("`group` requires an `id`, e.g. `{EXAMPLE}`")));
    };
    raw.groups.push(GroupDecl {
        id,
        required: group.required.is_some(),
        multiple: group.multiple.is_some(),
    });
    Ok(())
}
