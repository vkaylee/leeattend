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

    format!("{masked_local}{domain_part}")
}

pub fn mask_phone(phone: &str) -> String {
    let len = phone.len();
    if len <= 6 {
        return "***".to_string();
    }
    let first = &phone[..3];
    let last = &phone[len - 3..];
    let stars = "*".repeat(len - 6);
    format!("{first}{stars}{last}")
}

pub fn mask_bank_account(account: &str) -> String {
    let len = account.len();
    if len <= 4 {
        return "****".to_string();
    }
    let last = &account[len - 4..];
    let stars = "*".repeat(len - 4);
    format!("{stars}{last}")
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

    #[test]
    fn test_mask_phone() {
        assert_eq!(mask_phone("0987654321"), "098****321");
        assert_eq!(mask_phone("12345"), "***");
        assert_eq!(mask_phone("1234567"), "123*567");
    }

    #[test]
    fn test_mask_bank_account() {
        assert_eq!(mask_bank_account("1234567890"), "******7890");
        assert_eq!(mask_bank_account("123"), "****");
        assert_eq!(mask_bank_account("12345"), "*2345");
    }
}
