pub(super) fn to_kebab_case(name: &str) -> String {
    let characters: Vec<char> = name.chars().collect();
    let mut output = String::new();
    for (index, &character) in characters.iter().enumerate() {
        if character == '_' {
            if !output.is_empty() && !output.ends_with('-') {
                output.push('-');
            }
            continue;
        }
        if character.is_uppercase() && !output.is_empty() && !output.ends_with('-') {
            let previous = characters[index - 1];
            let next_is_lowercase = characters
                .get(index + 1)
                .is_some_and(|next| next.is_lowercase());
            if previous.is_lowercase()
                || previous.is_ascii_digit()
                || (previous.is_uppercase() && next_is_lowercase)
            {
                output.push('-');
            }
        }
        output.extend(character.to_lowercase());
    }
    output.trim_end_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::to_kebab_case;

    #[test]
    fn converts_variant_names() {
        assert_eq!(to_kebab_case("Fast"), "fast");
        assert_eq!(to_kebab_case("DryRun"), "dry-run");
        assert_eq!(to_kebab_case("HTTPServer"), "http-server");
        assert_eq!(to_kebab_case("Utf8Text"), "utf8-text");
        assert_eq!(to_kebab_case("Snake_Case"), "snake-case");
        assert_eq!(to_kebab_case("ABC"), "abc");
    }
}
