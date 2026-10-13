use super::container::GroupDecl;
use super::errors::Errors;
use super::field::{FieldKind, FieldModel};
use std::collections::HashSet;
use syn::LitStr;

pub(crate) struct ArgGroup<'a> {
    pub(crate) id: String,
    pub(crate) required: bool,
    pub(crate) multiple: bool,
    pub(crate) members: Vec<&'a FieldModel>,
}

pub(crate) fn resolve_groups<'a>(
    declarations: &[GroupDecl],
    fields: &'a [FieldModel],
) -> Vec<ArgGroup<'a>> {
    let mut groups: Vec<ArgGroup> = Vec::new();
    for declaration in declarations {
        let id = declaration.id.value();
        if groups.iter().all(|group| group.id != id) {
            groups.push(ArgGroup {
                id,
                required: declaration.required,
                multiple: declaration.multiple,
                members: Vec::new(),
            });
        }
    }
    for field in fields {
        let Some(literal) = &field.relations.group else {
            continue;
        };
        let id = literal.value();
        match groups.iter_mut().find(|group| group.id == id) {
            Some(group) => group.members.push(field),
            None => groups.push(ArgGroup {
                id,
                required: false,
                multiple: false,
                members: vec![field],
            }),
        }
    }
    groups
}

fn pair<'a>(first: &'a str, second: &'a str) -> (&'a str, &'a str) {
    match first <= second {
        true => (first, second),
        false => (second, first),
    }
}

fn references<'a>(
    field: &FieldModel,
    key: &str,
    ids: &'a [LitStr],
    names: &HashSet<&str>,
    errors: &mut Errors,
) -> Vec<(&'a LitStr, String)> {
    let mut valid: Vec<(&LitStr, String)> = Vec::new();
    for literal in ids {
        let id = literal.value();
        if id == field.name {
            errors.push(syn::Error::new(
                literal.span(),
                format!("`{key}` cannot reference the field `{id}` itself"),
            ));
        } else if !names.contains(id.as_str()) {
            errors.push(syn::Error::new(
                literal.span(),
                format!("`{key}` references unknown argument `{id}`; expected the name of a field of this struct"),
            ));
        } else if valid.iter().any(|(_, existing)| *existing == id) {
            errors.push(syn::Error::new(
                literal.span(),
                format!("duplicate `{id}` in `{key}`"),
            ));
        } else {
            valid.push((literal, id));
        }
    }
    valid
}

pub(super) fn validate_relations(
    declarations: &[GroupDecl],
    fields: &[FieldModel],
    errors: &mut Errors,
) {
    let names: HashSet<&str> = fields.iter().map(|field| field.name.as_str()).collect();
    let mut exclusive: HashSet<(String, String)> = HashSet::new();
    let mut requirements: Vec<(&FieldModel, &LitStr, String)> = Vec::new();

    for field in fields {
        let relations = &field.relations;
        for (_, id) in references(
            field,
            "conflicts_with",
            &relations.conflicts_with,
            &names,
            errors,
        ) {
            let (first, second) = pair(&field.name, &id);
            exclusive.insert((first.to_owned(), second.to_owned()));
        }
        for (literal, id) in references(field, "requires", &relations.requires, &names, errors) {
            requirements.push((field, literal, id));
        }
        references(
            field,
            "required_unless_present",
            &relations.required_unless_present,
            &names,
            errors,
        );
        if let Some(literal) = relations.required_unless_present.first()
            && !field.is_optional()
            && !matches!(field.kind, FieldKind::Flag(_))
        {
            errors.push(syn::Error::new(
                literal.span(),
                format!(
                    "`required_unless_present` requires an optional field; change the type of `{}` to `Option<T>` or give it a default",
                    field.name
                ),
            ));
        }
    }

    let mut declared: HashSet<String> = HashSet::new();
    for declaration in declarations {
        let id = declaration.id.value();
        if !declared.insert(id.clone()) {
            errors.push(syn::Error::new(
                declaration.id.span(),
                format!("duplicate group `{id}`"),
            ));
        } else if fields
            .iter()
            .all(|field| field.relations.group.as_ref().map(LitStr::value) != Some(id.clone()))
        {
            errors.push(syn::Error::new(
                declaration.id.span(),
                format!(
                    "group `{id}` has no arguments; add `#[arg(group = \"{id}\")]` to its fields"
                ),
            ));
        }
    }

    for group in resolve_groups(declarations, fields) {
        if group.multiple {
            continue;
        }
        for (index, first) in group.members.iter().enumerate() {
            for second in &group.members[index + 1..] {
                let (first, second) = pair(&first.name, &second.name);
                exclusive.insert((first.to_owned(), second.to_owned()));
            }
        }
    }

    for (field, literal, id) in requirements {
        let (first, second) = pair(&field.name, &id);
        if exclusive.contains(&(first.to_owned(), second.to_owned())) {
            errors.push(syn::Error::new(
                literal.span(),
                format!(
                    "`{}` cannot require `{id}` because the two arguments conflict with each other",
                    field.name
                ),
            ));
        }
    }
}
