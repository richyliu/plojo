use regex::{Regex, RegexBuilder};
use std::collections::HashSet;

lazy_static! {
    static ref ORTHOGRAPHY_RULES: Rules = default_orthography();
    static ref ORTHOGRAPHY_DICT: HashSet<String> = load_orthography_dict();
}

fn default_orthography() -> Rules {
    // helper for building rules
    fn rule_with_lit(
        b: &str,
        s: &str,
        lit: &'static str,
        keep_suffix_group: bool,
    ) -> (Find, Replace) {
        (
            Find::new(b, s),
            if keep_suffix_group {
                vec![
                    ReplaceItem::BaseGroup(1),
                    ReplaceItem::Lit(lit),
                    ReplaceItem::SuffixGroup(1),
                ]
            } else {
                vec![ReplaceItem::BaseGroup(1), ReplaceItem::Lit(lit)]
            },
        )
    }

    // Same orthography rules as Plover
    // Source: https://github.com/opensteno/plover/commit/ccea856aa75723047eb972b8cbb6ddaf471d3ae6
    vec![
        // artistic + ly = artistically
        rule_with_lit(r"^(.*[aeiou]c)$", r"^ly$", "ally", false),
        // humble + ly = humbly (*humblely)
        // questionable + ly = questionably
        // triple + ly = triply
        rule_with_lit(r"^(.+[aeioubmnp])le$", r"^ly$", "ly", false),
        // statute + ry = statutory
        rule_with_lit(r"^(.*t)e$", r"^(ry|ary)$", "ory", false),
        // confirm + tory = confirmatory (*confirmtory)
        rule_with_lit(r"^(.*)m$", r"^tor(y|ily)$", "mator", true),
        // supervise + ary = supervisory (*supervisary)
        rule_with_lit(r"^(.+)se$", r"^ar(y|ies)$", "sor", true),
        // frequent + cy = frequency (tcy/tecy removal)
        rule_with_lit(r"^(.*[naeiou])te?$", r"^cy$", "cy", false),
        // establish + s = establishes (sibilant pluralization)
        rule_with_lit(r"^(.*(?:s|sh|x|z|zh))$", r"^s$", "es", false),
        // speech + s = speeches (soft ch pluralization)
        // NOTE: removed ?<! because look-arounds are not supported
        rule_with_lit(
            r"^(.*(?:oa|ea|i|ee|oo|au|ou|l|n|[gin]ar|t)ch)$",
            r"^s$",
            "es",
            false,
        ),
        // cherry + s = cherries (consonant + y pluralization)
        rule_with_lit(r"^(.+[bcdfghjklmnpqrstvwxz])y$", r"^s$", "ies", false),
        // die+ing = dying
        rule_with_lit(r"^(.+)ie$", r"^ing$", "ying", false),
        // metallurgy + ist = metallurgist
        rule_with_lit(r"^(.+[cdfghlmnpr])y$", r"^ist$", "ist", false),
        // beauty + ful = beautiful (y -> i)
        rule_with_lit(
            r"^(.+[bcdfghjklmnpqrstvwxz])y$",
            r"^([a-hj-xz].*)$",
            "i",
            true,
        ),
        // write + en = written
        rule_with_lit(r"^(.+)te$", r"^en$", "tten", false),
        // Minnesota + en = Minnesotan (*Minnesotaen)
        rule_with_lit(r"^(.+[ae])$", r"^e(n|ns)$", "", true),
        // ceremony + ial = ceremonial (*ceremonyial)
        rule_with_lit(r"^(.+)y$", r"^(ial|ially)$", "", true),
        // spaghetti + ification = spaghettification (*spaghettiification)
        rule_with_lit(
            r"^(.+)i$",
            r"^if(y|ying|ied|ies|ication|ications)$",
            "if",
            true,
        ),
        // fantastic + ical = fantastical (*fantasticcal)
        rule_with_lit(r"^(.+)ic$", r"^ical$", "ical", false),
        // epistomology + ical = epistomological
        rule_with_lit(r"^(.+)ology$", r"^ic(al|ally)$", "ologic", true),
        // oratory + ical = oratorical (*oratoryical)
        rule_with_lit(r"^(.*)ry$", r"^ica(l|lly|lity)$", "rica", true),
        // radical + ist = radicalist (*radicallist)
        rule_with_lit(r"^(.*l)$", r"^is(t|ts)$", "is", true),
        // complementary + ity = complementarity (*complementaryity)
        rule_with_lit(r"^(.*)ry$", r"^ity$", "rity", false),
        // disproportional + ity = disproportionality (*disproportionallity)
        rule_with_lit(r"^(.+)l$", r"^ity$", "lity", false),
        // perform + tive = performative (*performtive)
        rule_with_lit(r"^(.+)rm$", r"^tiv(e|ity|ities)$", "rmativ", true),
        // restore + tive = restorative
        rule_with_lit(r"^(.+)e$", r"^tiv(e|ity|ities)$", "ativ", true),
        // token + ize / ise = tokenize / tokenise (*tokennize / *tokennise)
        rule_with_lit(
            r"^(.+)y$",
            r"^iz(e|es|ing|ed|er|ers|ation|ations|able|ability)$",
            "iz",
            true,
        ),
        rule_with_lit(
            r"^(.+)y$",
            r"^is(e|es|ing|ed|er|ers|ation|ations|able|ability)$",
            "is",
            true,
        ),
        // conditional + ize / ise = conditionalize / conditionalise (*conditionallize)
        rule_with_lit(
            r"^(.+)al$",
            r"^iz(e|ed|es|ing|er|ers|ation|ations|m|ms|able|ability|abilities)$",
            "aliz",
            true,
        ),
        rule_with_lit(
            r"^(.+)al$",
            r"^is(e|ed|es|ing|er|ers|ation|ations|m|ms|able|ability|abilities)$",
            "alis",
            true,
        ),
        // spectacular + ization = spectacularization (*spectacularrization)
        rule_with_lit(
            r"^(.+)ar$",
            r"^iz(e|ed|es|ing|er|ers|ation|ations|m|ms)$",
            "ariz",
            true,
        ),
        rule_with_lit(
            r"^(.+)ar$",
            r"^is(e|ed|es|ing|er|ers|ation|ations|m|ms)$",
            "aris",
            true,
        ),
        // category + ize / ise = categorize / categorise
        // custom + izable / isable = customizable / customisable
        // fantasy + ize = fantasize
        rule_with_lit(
            r"^(.*[lmnty])$",
            r"^iz(e|es|ing|ed|er|ers|ation|ations|m|ms|able|ability|abilities)$",
            "iz",
            true,
        ),
        rule_with_lit(
            r"^(.*[lmnty])$",
            r"^is(e|es|ing|ed|er|ers|ation|ations|m|ms|able|ability|abilities)$",
            "is",
            true,
        ),
        // criminal + ology = criminology; criminal + ologist = criminologist
        rule_with_lit(r"^(.+)al$", r"^olog(y|ist|ists|ical|ically)$", "olog", true),
        // similar + ish = similarish (*similarrish)
        (
            Find::new(r"^(.+)(ar|er|or)$", r"^ish$"),
            vec![
                ReplaceItem::BaseGroup(1),
                ReplaceItem::BaseGroup(2),
                ReplaceItem::Lit("ish"),
            ],
        ),
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
    if ORTHOGRAPHY_DICT.contains(&simple_join.to_lowercase()) {
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
    fn test_orthography_plover_rules_update() {
        assert_eq!(orthog(vec!["humble", "ly"]), "humbly");
        assert_eq!(orthog(vec!["questionable", "ly"]), "questionably");
        assert_eq!(orthog(vec!["triple", "ly"]), "triply");
        assert_eq!(orthog(vec!["statute", "ary"]), "statutory");
        assert_eq!(orthog(vec!["confirm", "tory"]), "confirmatory");
        assert_eq!(orthog(vec!["supervise", "ary"]), "supervisory");
        assert_eq!(orthog(vec!["Minessota", "en"]), "Minessotan");
        assert_eq!(orthog(vec!["ceremony", "ial"]), "ceremonial");
        assert_eq!(orthog(vec!["editor", "ial"]), "editorial");
        assert_eq!(orthog(vec!["editor", "ially"]), "editorially");
        assert_eq!(orthog(vec!["spaghetti", "ification"]), "spaghettification");
        assert_eq!(orthog(vec!["fantastic", "ical"]), "fantastical");
        assert_eq!(orthog(vec!["epistomology", "ical"]), "epistomological");
        assert_eq!(orthog(vec!["oratory", "ical"]), "oratorical");
        assert_eq!(orthog(vec!["radical", "ist"]), "radicalist");
        assert_eq!(orthog(vec!["complementary", "ity"]), "complementarity");
        assert_eq!(orthog(vec!["disproportional", "ity"]), "disproportionality");
        assert_eq!(orthog(vec!["perform", "tive"]), "performative");
        assert_eq!(orthog(vec!["restore", "tive"]), "restorative");
        assert_eq!(orthog(vec!["token", "ize"]), "tokenize");
        assert_eq!(orthog(vec!["token", "ise"]), "tokenise");
        assert_eq!(orthog(vec!["conditional", "ize"]), "conditionalize");
        assert_eq!(orthog(vec!["conditional", "isation"]), "conditionalisation");
        assert_eq!(orthog(vec!["spectacular", "ization"]), "spectacularization");
        assert_eq!(orthog(vec!["spectacular", "ism"]), "spectacularism");
        assert_eq!(orthog(vec!["category", "ize"]), "categorize");
        assert_eq!(orthog(vec!["category", "ise"]), "categorise");
        assert_eq!(orthog(vec!["custom", "izable"]), "customizable");
        assert_eq!(orthog(vec!["custom", "isable"]), "customisable");
        assert_eq!(orthog(vec!["fantasy", "ize"]), "fantasize");
        assert_eq!(orthog(vec!["criminal", "ology"]), "criminology");
        assert_eq!(orthog(vec!["criminal", "ologist"]), "criminologist");
        assert_eq!(orthog(vec!["dermatology", "ist"]), "dermatologist");
        assert_eq!(orthog(vec!["similar", "ish"]), "similarish");
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
        assert_eq!(orthog(vec!["SHiver", "ing"]), "SHivering");
        assert_eq!(orthog(vec!["sHivER", "iNG"]), "sHivERiNG");
    }
}
