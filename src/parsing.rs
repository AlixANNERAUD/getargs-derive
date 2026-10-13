use core::cell::Cell;

fn is_number(text: &str) -> bool {
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(index) => (&text[..index], Some(&text[index + 1..])),
        None => (text, None),
    };
    let mut digits = 0_usize;
    let mut dot = false;
    for byte in mantissa.bytes() {
        match byte {
            b'0'..=b'9' => digits += 1,
            b'.' if !dot && digits > 0 => dot = true,
            _ => return false,
        }
    }
    if digits == 0 {
        return false;
    }
    match exponent {
        None => true,
        Some(exponent) => {
            let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
            !exponent.is_empty() && exponent.bytes().all(|byte| byte.is_ascii_digit())
        }
    }
}

fn is_negative_number(value: &str) -> bool {
    value.strip_prefix('-').is_some_and(is_number)
}

#[doc(hidden)]
pub fn accepts_value(value: &str, allow_negative_numbers: bool) -> bool {
    !value.starts_with('-') || value == "-" || (allow_negative_numbers && is_negative_number(value))
}

#[doc(hidden)]
pub fn negative_number<'a>(raw: Option<&Cell<Option<&'a str>>>, short: char) -> Option<&'a str> {
    let raw = raw?.get()?;
    let rest = raw.strip_prefix('-')?;
    (rest.starts_with(short) && is_number(rest)).then_some(raw)
}

#[doc(hidden)]
pub struct RawArgs<'r, 'a, I> {
    iter: I,
    last: &'r Cell<Option<&'a str>>,
}

impl<'r, 'a, I> RawArgs<'r, 'a, I> {
    #[doc(hidden)]
    pub fn new(iter: I, last: &'r Cell<Option<&'a str>>) -> Self {
        Self { iter, last }
    }
}

impl<'a, I: Iterator<Item = &'a str>> Iterator for RawArgs<'_, 'a, I> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let item = self.iter.next();
        self.last.set(item);
        item
    }
}
