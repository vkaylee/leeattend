pub fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        // If it's not a valid email, just mask everything except first 2 chars
        let len = email.len();
        if len <= 2 {
            return "***".to_string();
        }
        return format!("{}***", &email[..2]);
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    let masked_local = if local_part.len() <= 2 {
        "***".to_string()
    } else if local_part.len() <= 4 {
        format!("{}***", &local_part[..1])
    } else {
        format!("{}***", &local_part[..3])
    } + "@";

    format!("{}{}", masked_local, domain_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("john.doe@example.com"), "joh***@example.com");
        assert_eq!(mask_email("lee@gmail.com"), "l***@gmail.com");
        assert_eq!(mask_email("me@test.com"), "***@test.com");
        assert_eq!(mask_email("a@b.com"), "***@b.com");
        assert_eq!(mask_email("longemailaddress@corp.com"), "lon***@corp.com");
    }
}
