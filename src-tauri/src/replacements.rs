use crate::storage::config::ReplacementRule;
use regex::Regex;

pub fn apply(input: &str, rules: &[ReplacementRule]) -> String {
    let mut s = input.to_string();
    for r in rules {
        if r.regex {
            if let Ok(re) = Regex::new(&r.from) {
                s = re.replace_all(&s, r.to.as_str()).to_string();
            }
        } else {
            s = s.replace(&r.from, &r.to);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::config::ReplacementRule;

    fn lit(from: &str, to: &str) -> ReplacementRule {
        ReplacementRule {
            from: from.into(),
            to: to.into(),
            regex: false,
        }
    }
    fn rx(from: &str, to: &str) -> ReplacementRule {
        ReplacementRule {
            from: from.into(),
            to: to.into(),
            regex: true,
        }
    }

    #[test]
    fn literal_replace() {
        assert_eq!(apply("hello world", &[lit("hello", "hi")]), "hi world");
    }

    #[test]
    fn regex_word_boundary() {
        assert_eq!(
            apply("send mail to mailbox", &[rx(r"\bmail\b", "email")]),
            "send email to mailbox"
        );
    }

    #[test]
    fn invalid_regex_is_skipped() {
        assert_eq!(apply("ok", &[rx("[", "x")]), "ok");
    }

    #[test]
    fn rules_applied_in_order() {
        let rules = vec![lit("a", "b"), lit("b", "c")];
        assert_eq!(apply("a", &rules), "c");
    }
}
