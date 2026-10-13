use syn::{GenericArgument, PathArguments, Type};

pub(super) fn is_bool_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == "bool")
            .unwrap_or(false),
        _ => false,
    }
}

pub(super) fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    match arguments.args.first() {
        Some(GenericArgument::Type(inner)) if arguments.args.len() == 1 => Some(inner),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    Str,
    Bytes,
    OsStr,
    Path,
    OsOwned,
    Parsed,
}

impl ValueKind {
    pub(crate) fn requires_std(self) -> bool {
        matches!(self, ValueKind::OsStr | ValueKind::Path)
    }
}

fn last_ident(ty: &Type) -> Option<&syn::Ident> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            path.path.segments.last().map(|segment| &segment.ident)
        }
        _ => None,
    }
}

pub(super) fn value_kind(ty: &Type) -> ValueKind {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Slice(slice) if last_ident(&slice.elem).is_some_and(|ident| ident == "u8") => {
                ValueKind::Bytes
            }
            elem => match last_ident(elem) {
                Some(ident) if ident == "str" => ValueKind::Str,
                Some(ident) if ident == "OsStr" => ValueKind::OsStr,
                Some(ident) if ident == "Path" => ValueKind::Path,
                _ => ValueKind::Parsed,
            },
        },
        _ => match last_ident(ty) {
            Some(ident) if cfg!(feature = "std") && (ident == "PathBuf" || ident == "OsString") => {
                ValueKind::OsOwned
            }
            _ => ValueKind::Parsed,
        },
    }
}

const COUNT_TYPES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
];

pub(super) fn is_count_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            path.qself.is_none()
                && path.path.segments.last().is_some_and(|segment| {
                    segment.arguments.is_none()
                        && COUNT_TYPES.iter().any(|name| segment.ident == name)
                })
        }
        _ => false,
    }
}

const COLLECTION_TYPES: &[&str] = &["Values", "Vec"];

pub(super) fn collection_element(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    if !COLLECTION_TYPES.iter().any(|name| segment.ident == name) {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    match arguments.args.first() {
        Some(GenericArgument::Type(element)) => Some(element),
        _ => None,
    }
}
