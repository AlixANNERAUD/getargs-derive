use syn::{Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue};

fn doc_lines(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(literal),
                        ..
                    }),
                ..
            }) => Some(literal.value()),
            _ => None,
        })
        .flat_map(|value| {
            value
                .split('\n')
                .map(|line| line.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) fn short_doc(attrs: &[Attribute]) -> Option<String> {
    let lines = doc_lines(attrs);
    let paragraph: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .skip_while(|line| line.is_empty())
        .take_while(|line| !line.is_empty())
        .collect();
    if paragraph.is_empty() {
        return None;
    }
    let mut text = paragraph.join(" ");
    if text.ends_with('.') && !text.ends_with("..") {
        text.pop();
    }
    Some(text)
}
