use regex::{Regex, RegexSet};

pub struct SwedishScrubber {
    regex_set: RegexSet,
    patterns: Vec<Regex>,
    replacements: Vec<&'static str>,
}

impl SwedishScrubber {
    pub fn new() -> Self {
        // 1. Personnummer: r"(?x) \b (19|20)? \d{2} (0[1-9]|1[0-2]) ([0-36-9]\d) [-+]? \d{4} \b"
        // 2. Bankgiro: r"\b\d{3,4}-\d{4}\b"
        // 3. Car Plates: r"\b[A-HJ-PR-UW-Z]{3}[\s-]?\d{2}[A-Z0-9]\b"
        // 4. Mobile (+467/07): r"(\+46|0)7[02369][\s-]?\d{2}[\s-]?\d{2}[\s-]?\d{3}"
        // 5. Credit Cards: r"\b(?:\d{4}[- ]?){3}\d{4}\b"

        let pattern_strs = vec![
            r"(?x) \b (19|20)? \d{2} (0[1-9]|1[0-2]) ([0-36-9]\d) [-+]? \d{4} \b",
            r"\b\d{3,4}-\d{4}\b",
            r"\b[A-HJ-PR-UW-Z]{3}[\s-]?\d{2}[A-Z0-9]\b",
            r"(\+46|0)7[02369](?:[\s-]?\d){7}\b",
            r"\b(?:\d{4}[- ]?){3}\d{4}\b",
        ];

        let replacements = vec![
            "<SE_PERSONNUMMER>",
            "<SE_BANKGIRO>",
            "<SE_CAR_PLATE>",
            "<SE_MOBILE>",
            "<CREDIT_CARD>",
        ];

        let regex_set = RegexSet::new(&pattern_strs).expect("Failed to create RegexSet");
        let patterns = pattern_strs
            .iter()
            .map(|s| Regex::new(s).expect("Failed to compile regex"))
            .collect();

        Self {
            regex_set,
            patterns,
            replacements,
        }
    }

    pub fn scrub(&self, text: &str) -> (String, usize) {
        // Optimization: Use RegexSet to check if any pattern matches before iterating
        if !self.regex_set.is_match(text) {
            return (text.to_string(), 0);
        }

        let mut result = text.to_string();
        let mut total_replacements = 0;

        for (i, pattern) in self.patterns.iter().enumerate() {
            if self.regex_set.matches(text).matched(i) {
                // We need to count replacements. replace_all doesn't return count directly in regex 1.x (Cow).
                // But we can check matches count or just accept slight inefficiency if we want exact count.
                // Or use `replace_all` and check if string changed?
                // Simplest consistent way: 
                // total_replacements += pattern.find_iter(&result).count(); 
                // result = pattern.replace_all(&result, self.replacements[i]).to_string(); 
                // But pattern matching on `result` which changes... standard is iterating patterns.
                
                // Let's count first on the current state of result
                let count = pattern.find_iter(&result).count();
                if count > 0 {
                    total_replacements += count;
                    result = pattern
                        .replace_all(&result, self.replacements[i])
                        .to_string();
                }
            }
        }

        (result, total_replacements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub_personnummer() {
        let scrubber = SwedishScrubber::new();
        let input = "My ID is 19900101-1234.";
        let expected = "My ID is <SE_PERSONNUMMER>.";
        let (res, count) = scrubber.scrub(input);
        assert_eq!(res, expected);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_scrub_bankgiro() {
        let scrubber = SwedishScrubber::new();
        let input = "Pay to 123-4567 please.";
        let expected = "Pay to <SE_BANKGIRO> please.";
        let (res, count) = scrubber.scrub(input);
        assert_eq!(res, expected);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_scrub_car_plate() {
        let scrubber = SwedishScrubber::new();
        let input = "Car ABC 123 parked here.";
        let expected = "Car <SE_CAR_PLATE> parked here.";
        let (res, count) = scrubber.scrub(input);
        assert_eq!(res, expected);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_scrub_mobile() {
        let scrubber = SwedishScrubber::new();
        let input = "Call me at 070-123 45 67.";
        let expected = "Call me at <SE_MOBILE>.";
        let (res, count) = scrubber.scrub(input);
        assert_eq!(res, expected);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_scrub_credit_card() {
        let scrubber = SwedishScrubber::new();
        let input = "Card: 1234 5678 1234 5678.";
        let expected = "Card: <CREDIT_CARD>.";
        let (res, count) = scrubber.scrub(input);
        assert_eq!(res, expected);
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_no_scrub() {
        let scrubber = SwedishScrubber::new();
        let input = "Just some normal text with number 12345.";
        let expected = "Just some normal text with number 12345.";
        let (res, count) = scrubber.scrub(input);
        assert_eq!(res, expected);
        assert_eq!(count, 0);
    }
}
