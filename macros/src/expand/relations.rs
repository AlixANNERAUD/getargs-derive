use crate::model::{ContainerModel, FieldModel, resolve_groups};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn present(field: &FieldModel) -> TokenStream {
    let value_ident = format_ident!("__value_{}", field.name);
    quote! { #value_ident.is_some() }
}

fn find<'a>(fields: &'a [FieldModel], id: &str) -> &'a FieldModel {
    fields
        .iter()
        .find(|field| field.name == id)
        .expect("relations reference fields validated by the model")
}

fn conflict(first: &FieldModel, second: &FieldModel) -> TokenStream {
    let first_present = present(first);
    let second_present = present(second);
    let name = first.error_name();
    let other = second.error_name();
    quote! {
        if #first_present && #second_present {
            return Err(getargs_derive::Error::ArgumentConflict {
                name: #name,
                other: #other,
            });
        }
    }
}

fn one_of(members: &[&FieldModel]) -> TokenStream {
    let present = members.iter().map(|field| present(field));
    let names = members.iter().map(|field| field.error_name());
    quote! {
        if !(#(#present)||*) {
            return Err(getargs_derive::Error::MissingOneOf(&[#(#names),*]));
        }
    }
}

pub(super) fn relation_checks(container: &ContainerModel, fields: &[FieldModel]) -> TokenStream {
    let groups = resolve_groups(&container.groups, fields);
    let mut conflicts = Vec::new();
    let mut required = Vec::new();
    let mut requirements = Vec::new();

    for field in fields {
        let relations = &field.relations;
        for id in &relations.conflicts_with {
            conflicts.push(conflict(field, find(fields, &id.value())));
        }
        if !relations.required_unless_present.is_empty() {
            let mut members = vec![field];
            members.extend(
                relations
                    .required_unless_present
                    .iter()
                    .map(|id| find(fields, &id.value())),
            );
            required.push(one_of(&members));
        }
        for id in &relations.requires {
            let other = find(fields, &id.value());
            let field_present = present(field);
            let other_present = present(other);
            let name = other.error_name();
            let required_by = field.error_name();
            requirements.push(quote! {
                if #field_present && !#other_present {
                    return Err(getargs_derive::Error::MissingRequiredArgument {
                        name: #name,
                        required_by: #required_by,
                    });
                }
            });
        }
    }

    for group in &groups {
        if !group.multiple {
            for (index, first) in group.members.iter().enumerate() {
                for second in &group.members[index + 1..] {
                    conflicts.push(conflict(first, second));
                }
            }
        }
        if group.required {
            required.push(one_of(&group.members));
        }
    }

    quote! {
        #(#conflicts)*
        #(#required)*
        #(#requirements)*
    }
}
