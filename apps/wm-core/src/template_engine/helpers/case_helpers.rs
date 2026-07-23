pub fn apply_helper(name: &str, value: &str) -> String {
    match name {
        "pascalCase" | "PascalCase" => to_pascal_case(value),
        "camelCase" => to_camel_case(value),
        "kebabCase" | "kebab-case" => to_kebab_case(value),
        "snakeCase" | "snake_case" => to_snake_case(value),
        "upperCase" | "UPPERCASE" => value.to_uppercase(),
        "lowerCase" | "lowercase" => value.to_lowercase(),
        "startCase" | "Start Case" => to_start_case(value),
        _ => value.to_string(),
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut next_upper = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            next_upper = true;
        } else if next_upper {
            result.push(ch.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut result = pascal;
    if let Some(c) = result.get_mut(..1) {
        c.make_ascii_lowercase();
    }
    result
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == ' ' {
            result.push('_');
        } else {
            result.push(ch.to_ascii_lowercase());
        }
    }
    result
}

fn to_start_case(s: &str) -> String {
    let mut result = String::new();
    let mut next_upper = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            result.push(' ');
            next_upper = true;
        } else if next_upper {
            result.push(ch.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(ch);
        }
    }
    result
}
