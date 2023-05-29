pub fn dim_if(s: String, cond: bool) -> String {
    if cond {
        format!("<span alpha=\"50%\">{s}</span>")
    } else {
        s
    }
}

pub fn dim_leading_zeros(s: String) -> String {
    if !s.starts_with("0") {
        return s;
    }

    let mut zeros: usize = 0;
    for c in s.chars() {
        if c == '0' {
            zeros += 1;
        } else {
            break;
        }
    }

    let (zeros, rest) = s.split_at(zeros);
    format!("<span alpha=\"50%\">{zeros}</span>{rest}")
}

pub fn pad_with_dim_leading_zeros(s: String, max_length: usize) -> String {
    let zeros = "0".repeat(max_length.saturating_sub(s.len()));
    format!("<span alpha=\"50%\">{zeros}</span>{s}")
}
