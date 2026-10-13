use crate::model::{FieldKind, FieldModel};

pub(super) const ARGUMENTS: &str = "Arguments";
pub(super) const OPTIONS: &str = "Options";

pub(super) struct Section<'a> {
    pub(super) title: &'a str,
    pub(super) fields: Vec<&'a FieldModel>,
}

pub(super) fn sections(fields: &[FieldModel]) -> Vec<Section<'_>> {
    let mut sections: Vec<(Section, Vec<usize>)> = [ARGUMENTS, OPTIONS]
        .into_iter()
        .map(|title| {
            (
                Section {
                    title,
                    fields: Vec::new(),
                },
                Vec::new(),
            )
        })
        .collect();

    for (index, field) in fields.iter().enumerate().filter(|(_, field)| !field.hide) {
        let title = field.help_heading.as_deref().unwrap_or(match field.kind {
            FieldKind::Positional => ARGUMENTS,
            FieldKind::Flag(_) | FieldKind::Option => OPTIONS,
        });
        let position = match sections
            .iter()
            .position(|(section, _)| section.title == title)
        {
            Some(position) => position,
            None => {
                sections.push((
                    Section {
                        title,
                        fields: Vec::new(),
                    },
                    Vec::new(),
                ));
                sections.len() - 1
            }
        };
        let (section, orders) = &mut sections[position];
        section.fields.push(field);
        orders.push(field.display_order.unwrap_or(index));
    }

    sections
        .into_iter()
        .map(|(section, orders)| {
            let mut keyed: Vec<_> = orders.into_iter().zip(section.fields).collect();
            keyed.sort_by_key(|(order, _)| *order);
            Section {
                title: section.title,
                fields: keyed.into_iter().map(|(_, field)| field).collect(),
            }
        })
        .collect()
}
