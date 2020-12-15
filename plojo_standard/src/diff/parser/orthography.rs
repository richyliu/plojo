use regex::{Regex, RegexBuilder};
use std::collections::HashSet;

lazy_static! {
    static ref ORTHOGRAPHY_RULES: Rules = default_orthography();
    static ref ORTHOGRAPHY_DICT: HashSet<String> = load_orthography_dict();
}

fn default_orthography() -> Rules {
    // helper for building rules
    fn rule_with_lit(b: &str, s: &str, lit: &'static str) -> (Find, Replace) {
        (
            Find::new(b, s),
            vec![ReplaceItem::BaseGroup(1), ReplaceItem::Lit(lit)],
        )
    }

    // Same orthography rules as Plover
    // Source: https://github.com/openstenoproject/plover/blob/master/plover/system/english_stenotype.py
    vec![
        // artistic + ly = artistically
        rule_with_lit(r"^(.*[aeiou]c)$", r"^ly$", "ally"),
        // statute + ry = statutory
        rule_with_lit(r"^(.*t)e$", r"^ry$", "ory"),
        // frequent + cy = frequency (tcy/tecy removal)
        rule_with_lit(r"^(.*[naeiou])te?$", r"^cy$", "cy"),
        // establish + s = establishes (sibilant pluralization)
        rule_with_lit(r"^(.*(?:s|sh|x|z|zh))$", r"^s$", "es"),
        // speech + s = speeches (soft ch pluralization)
        // NOTE: removed ?<! because look-arounds are not supported
        rule_with_lit(
            r"^(.*(?:oa|ea|i|ee|oo|au|ou|l|n|[gin]ar|t)ch)$",
            r"^s$",
            "es",
        ),
        // cherry + s = cherries (consonant + y pluralization)
        rule_with_lit(r"^(.+[bcdfghjklmnpqrstvwxz])y$", r"^s$", "ies"),
        // die+ing = dying
        rule_with_lit(r"^(.+)ie$", r"^ing$", "ying"),
        // metallurgy + ist = metallurgist
        rule_with_lit(r"^(.+[cdfghlmnpr])y$", r"^ist$", "ist"),
        // beauty + ful = beautiful (y -> i)
        (
            Find::new(r"^(.+[bcdfghjklmnpqrstvwxz])y$", "^([a-hj-xz].*)$"),
            vec![
                ReplaceItem::BaseGroup(1),
                ReplaceItem::Lit("i"),
                ReplaceItem::SuffixGroup(1),
            ],
        ),
        // write + en = written
        rule_with_lit(r"^(.+)te$", r"^en$", "tten"),
        // free + ed = freed
        (
            Find::new(r"^(.+e)e$", "^(e.+)$"),
            vec![ReplaceItem::BaseGroup(1), ReplaceItem::SuffixGroup(1)],
        ),
        // narrate + ing = narrating (silent e)
        (
            Find::new(r"^(.+[bcdfghjklmnpqrstuvwxz])e$", "^([aeiouy].*)$"),
            vec![ReplaceItem::BaseGroup(1), ReplaceItem::SuffixGroup(1)],
        ),
        // defer + ed = deferred (consonant doubling)   XXX monitor(stress not on last syllable)
        (
            Find::new(
                r"^(.*(?:[bcdfghjklmnprstvwxyz]|qu)[aeiou])([bcdfgklmnprtvz])$",
                "^([aeiouy].*)$",
            ),
            vec![
                ReplaceItem::BaseGroup(1),
                ReplaceItem::BaseGroup(2),
                ReplaceItem::BaseGroup(2),
                ReplaceItem::SuffixGroup(1),
            ],
        ),
    ]
}

fn load_orthography_dict() -> HashSet<String> {
    let mut set = HashSet::new();
    let raw_dict = include_str!("american_english_words.txt");

    for word in raw_dict.lines() {
        set.insert(word.to_string());
    }

    set
}

/// If a word and its suffix matches Find, it will be replaced with Replace
type Rules = Vec<(Find, Replace)>;

#[derive(Debug)]
struct Find {
    base: Regex,
    suffix: Regex,
}

impl Find {
    /// Creates a new find orthography rule with base and suffix regex
    /// Panics if either regex is invalid
    fn new(base_rule: &str, suffix_rule: &str) -> Self {
        Self {
            base: RegexBuilder::new(base_rule)
                .case_insensitive(true)
                .build()
                .unwrap(),
            suffix: RegexBuilder::new(suffix_rule)
                .case_insensitive(true)
                .build()
                .unwrap(),
        }
    }
}

impl PartialEq for Find {
    fn eq(&self, other: &Self) -> bool {
        self.base.as_str() == other.base.as_str() && self.suffix.as_str() == other.suffix.as_str()
    }
}

type Replace = Vec<ReplaceItem>;

/// Replace with a capturing group from base/suffix, or a literal string
#[derive(Debug, PartialEq)]
enum ReplaceItem {
    BaseGroup(usize),
    SuffixGroup(usize),
    Lit(&'static str),
}

/// Join a word and suffix together, applying orthographic (spelling) rules
/// It will first try a simple join of the suffix and look it up in a list of words
/// Panics for invalid rules
pub fn apply_orthography(base: &str, suffix: &str) -> String {
    // Try matching a simple join first and see if that is an english word
    // This is done mainly for consonant doubling rule, which sometimes doubles a consonant even
    // when it doesn't need to.
    let simple_join = base.to_owned() + suffix;
    if ORTHOGRAPHY_DICT.contains(&simple_join) {
        return simple_join;
    }

    for (find, replace) in ORTHOGRAPHY_RULES.iter() {
        if let (Some(base_captures), Some(suffix_captures)) =
            (find.base.captures(base), find.suffix.captures(suffix))
        {
            let mut s = String::new();
            for r in replace {
                s.push_str(match r {
                    // using unwrap() is fine here, because we assume the rules are valid
                    ReplaceItem::BaseGroup(group) => base_captures.get(*group).unwrap().as_str(),
                    ReplaceItem::SuffixGroup(group) => {
                        suffix_captures.get(*group).unwrap().as_str()
                    }
                    ReplaceItem::Lit(str) => *str,
                });
            }
            return s;
        }
    }

    // unable to match an orthography rule, just return the simple join of the strokes
    simple_join
}

#[cfg(test)]
mod tests {
    use super::*;

    // helper function that calls apply_orthography
    fn orthog(strs: Vec<&str>) -> String {
        let mut iter = strs.iter();
        let mut str = iter.next().unwrap().to_string();
        for s in iter {
            str = apply_orthography(&str, s);
        }
        str
    }

    #[test]
    fn test_orthography_basic() {
        assert_eq!(orthog(vec!["artistic", "ly"]), "artistically");
        assert_eq!(orthog(vec!["statute", "ry"]), "statutory");
        assert_eq!(orthog(vec!["frequent", "cy"]), "frequency");
        assert_eq!(orthog(vec!["establish", "s"]), "establishes");
        assert_eq!(orthog(vec!["speech", "s"]), "speeches");
        assert_eq!(orthog(vec!["cherry", "s"]), "cherries");
        assert_eq!(orthog(vec!["die", "ing"]), "dying");
        assert_eq!(orthog(vec!["metallurgy", "ist"]), "metallurgist");
        assert_eq!(orthog(vec!["beauty", "ful"]), "beautiful");
        assert_eq!(orthog(vec!["write", "en"]), "written");
        assert_eq!(orthog(vec!["free", "ed"]), "freed");
        assert_eq!(orthog(vec!["narrate", "ing"]), "narrating");
        assert_eq!(orthog(vec!["defer", "ed"]), "deferred");
    }

    #[test]
    fn test_orthography_multiple() {
        assert_eq!(orthog(vec!["artistic", "ly", "s"]), "artisticallies");
        assert_eq!(orthog(vec!["bite", "ing", "s"]), "bitings");
        assert_eq!(orthog(vec!["combine", "ate", "or"]), "combinator");
    }

    #[test]
    fn test_orthography_simple_join() {
        assert_eq!(orthog(vec!["monitor", "ed"]), "monitored");
        assert_eq!(orthog(vec!["shiver", "ing"]), "shivering");
    }

    #[test]
    fn test_orthography_uppercase() {
        assert_eq!(orthog(vec!["Big", "er"]), "Bigger");
        assert_eq!(orthog(vec!["biG", "eR"]), "biGGeR");
    }
}
