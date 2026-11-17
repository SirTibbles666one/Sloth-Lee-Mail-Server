//! Spam is scored by rspamd and tagged with `X-Spam: Yes`; until now nothing acted on that tag, so every flagged message landed
//! in the recipient's inbox with a header no client renders. This module is the acting part: recognise the tag, and name the
//! system group that quarantines it.
//!
//! The group is identified by its TITLE rather than a flag on `Group`, because `users.bin` is positional bincode — a new field
//! would fail to deserialise for every account already on disk. "Junk" is unique enough in practice, and the alternative is a
//! migration that risks the whole mailbox store.

use std::collections::HashMap;

/// The group a spam-flagged message is filed into.
pub const JUNK_GROUP_TITLE: &str = "Junk";

/// Is this group title the system quarantine? Case- and whitespace-insensitive, because a member may have typed `junk` when they
/// created their own group and that is still the group they mean.
pub fn is_junk_group_title(title: &str) -> bool {
    title.trim().eq_ignore_ascii_case(JUNK_GROUP_TITLE)
}

/// Did rspamd flag this message? `milter_headers` writes `X-Spam: Yes` once the score crosses `add_header`; header keys are
/// case-insensitive on the wire and Postfix may normalise them, so both sides are compared without case.
pub fn is_spam_flagged(headers: &HashMap<String, String>) -> bool {
    headers
        .iter()
        .any(|(key, value)| key.trim().eq_ignore_ascii_case("x-spam") && value.trim().eq_ignore_ascii_case("yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect()
    }

    #[test]
    fn recognises_the_tag_rspamd_writes() {
        assert!(is_spam_flagged(&headers(&[("X-Spam", "Yes")])));
        assert!(is_spam_flagged(&headers(&[("x-spam", "yes")])));
        assert!(is_spam_flagged(&headers(&[("X-SPAM", " YES ")])));
    }

    #[test]
    fn a_message_without_the_tag_is_not_spam() {
        assert!(!is_spam_flagged(&headers(&[("Subject", "hello"), ("X-Spam-Status", "No")])));
        assert!(!is_spam_flagged(&HashMap::new()));
    }

    #[test]
    fn only_yes_quarantines_so_other_verdicts_are_not_swallowed() {
        // rspamd writes "No" and other verdicts; filing those would hide real mail.
        assert!(!is_spam_flagged(&headers(&[("X-Spam", "No")])));
        assert!(!is_spam_flagged(&headers(&[("X-Spam", "Maybe")])));
    }

    #[test]
    fn the_junk_group_is_matched_without_case_or_padding() {
        assert!(is_junk_group_title("Junk"));
        assert!(is_junk_group_title(" junk "));
        assert!(is_junk_group_title("JUNK"));
        assert!(!is_junk_group_title("Junk mail"));
        assert!(!is_junk_group_title(""));
    }
}
