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

pub(super) fn is_str_reference_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident == "str")
                .unwrap_or(false),
            _ => false,
        },
        _ => false,
    }
}
