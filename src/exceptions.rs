pub fn handle_kuru(word: &str) -> String {
    if word == "来る" {
        return "[来|く]る".to_string();
    }

    let tail = word.chars().skip(1).collect::<String>();

    let is_formal = word.starts_with("来ま");
    let is_te_form = word.starts_with("来て");
    let is_ta_form = word.starts_with("来た");
    if is_formal || is_te_form || is_ta_form {
        return format!("[来|き]{}", tail).to_string();
    }

    return format!("[来|こ]{}", tail).to_string();
}

pub fn handle_suru(word: &str) -> String {
    if word == "為る" {
        return "[為|す]る".to_string();
    }
    let tail = word.chars().skip(1).collect::<String>();

    let is_formal = word.starts_with("為ま");
    let is_te_form = word.starts_with("為て");
    let is_ta_form = word.starts_with("為た");
    let is_imperative = word.starts_with("為ろ");
    if is_formal || is_te_form || is_ta_form || is_imperative {
        return format!("[為|し]{}", tail).to_string();
    }

    return format!("[為|さ]{}", tail).to_string();
}
