use super::RawField;
use crate::model::attr::{require_value, set_once};
use quote::ToTokens;
use syn::meta::ParseNestedMeta;

const SUPPORTED_ACTIONS: &str = "`SetTrue`, `SetFalse`, `Count`";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlagAction {
    SetTrue,
    SetFalse,
    Count,
}

impl FlagAction {
    pub(super) fn name(self) -> &'static str {
        match self {
            FlagAction::SetTrue => "SetTrue",
            FlagAction::SetFalse => "SetFalse",
            FlagAction::Count => "Count",
        }
    }
}

fn parse_action(meta: &ParseNestedMeta) -> syn::Result<(FlagAction, syn::Path)> {
    let path: syn::Path = require_value(meta, "action", "action = Count")?.parse()?;
    let plain = path.leading_colon.is_none()
        && path
            .segments
            .iter()
            .all(|segment| segment.arguments.is_none());
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let name = match segments.as_slice() {
        [name] if plain => name.as_str(),
        [prefix, name] if plain && prefix == "ArgAction" => name.as_str(),
        _ => "",
    };
    let action = match name {
        "SetTrue" => FlagAction::SetTrue,
        "SetFalse" => FlagAction::SetFalse,
        "Count" => FlagAction::Count,
        _ => {
            let text = path.to_token_stream().to_string().replace(' ', "");
            return Err(syn::Error::new_spanned(
                &path,
                format!("unsupported action `{text}`; supported actions are: {SUPPORTED_ACTIONS}"),
            ));
        }
    };
    Ok((action, path))
}

pub(super) fn action(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = parse_action(meta)?;
    set_once(&mut raw.action, value, meta)
}
