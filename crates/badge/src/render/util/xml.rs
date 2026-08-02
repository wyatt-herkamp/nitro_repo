use std::sync::LazyLock;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, AhoCorasickKind, MatchKind};
use regex::Regex;

const XML_ESCAPE_PATTERNS: [&str; 5] = ["&", "<", ">", "\"", "'"];
const XML_ESCAPE_REPLACEMENTS: [&str; 5] = ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"];

const XML_STRIP_TRAILING_PATTERNS_LEN: usize = 8;
static XML_STRIP_TRAILING_PATTERNS: LazyLock<(Vec<String>, Vec<String>)> = LazyLock::new(|| {
    let a = aho_corasick_pattern_builder(XML_STRIP_TRAILING_PATTERNS_LEN, ">", " ");
    let b = aho_corasick_pattern_builder(XML_STRIP_TRAILING_PATTERNS_LEN, ">\n", " ");
    combine_builders(a, b)
});
static FILL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"fill=[^\s]+").expect("Failed to build regex"));
static STRIP_TRAILING: LazyLock<AhoCorasick> = LazyLock::new(|| {
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostFirst)
        .build(&XML_STRIP_TRAILING_PATTERNS.0)
        .expect("Failed to build AhoCorasick")
});
static ESCAPE_XML: LazyLock<AhoCorasick> = LazyLock::new(|| {
    AhoCorasickBuilder::new()
        .kind(Some(AhoCorasickKind::DFA))
        .build(XML_ESCAPE_PATTERNS)
        .expect("Failed to build AhoCorasick")
});
/// Finds the fill value defined or if not defined will add a default value.
/// # Arguments
/// `svg` - The svg to search.
/// `replace_with` - The value to replace the fill with.
pub(crate) fn replace_fill_attribute(svg: &str, replace_with: &str) -> String {
    if FILL_PATTERN.is_match(svg) {
        FILL_PATTERN.replace_all(svg, replace_with).to_string()
    } else {
        svg.replace("<svg", &format!("<svg {}", replace_with))
    }
}

#[test]
pub fn test_replace_fill_attribute() {
    assert_eq!(
        "<svg fill=\"#ffffff\"",
        replace_fill_attribute("<svg fill=\"#000000\"", "fill=\"#ffffff\"")
    );
    assert_eq!(
        "<svg fill=\"#ffffff\"",
        replace_fill_attribute("<svg", "fill=\"#ffffff\"")
    );
}

fn strip_xml_trailing_aho(s: &str) -> String {
    STRIP_TRAILING.replace_all(s, &XML_STRIP_TRAILING_PATTERNS.1)
}

pub(crate) fn escape_xml(s: &str) -> String {
    ESCAPE_XML.replace_all(s, &XML_ESCAPE_REPLACEMENTS)
}

pub(crate) fn strip_xml_whitespace(s: &str) -> String {
    strip_xml_trailing_aho(s)
}

fn aho_corasick_pattern_builder(
    size: usize,
    start: &str,
    onwards: &str,
) -> (Vec<String>, Vec<String>) {
    // todo see if new lines are generating on windows

    let mut matches = vec![];

    let mut building = format!("{}{}", start, onwards);
    for _ in 0..size {
        matches.push(building.to_string());
        building.push_str(onwards);
    }
    matches.reverse();

    (matches, vec![">".to_string(); size])
}

fn combine_builders(
    (mut a, mut b): (Vec<String>, Vec<String>),
    (mut aa, mut bb): (Vec<String>, Vec<String>),
) -> (Vec<String>, Vec<String>) {
    a.append(&mut aa);
    b.append(&mut bb);
    (a, b)
}

#[cfg(test)]
mod tests {
    use crate::render::util::xml::{escape_xml, strip_xml_whitespace};

    const SVG_TO_STRIP: &str = r###"<svg xmlns="http://www.w3.org/2000/svg" aria-label="i">
    <title>i </title>
    <linearGradient id="s" x2="0" y2="100%">
        <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
        <stop offset="1" stop-opacity=".1"/>
    </linearGradient>
     </svg>"###;

    #[test]
    fn escape_xml_test() {
        assert_eq!(
            escape_xml(r#"& this would < > " '' "#),
            r#"&amp; this would &lt; &gt; &quot; &apos;&apos; "#
        );

        assert_eq!(escape_xml(r#"hello"#), r#"hello"#);

        assert_eq!(
            escape_xml(r#"&&&t&&& ts' <wo>msf<<<<**"">> this would < > " '' "#),
            r#"&amp;&amp;&amp;t&amp;&amp;&amp; ts&apos; &lt;wo&gt;msf&lt;&lt;&lt;&lt;**&quot;&quot;&gt;&gt; this would &lt; &gt; &quot; &apos;&apos; "#
        );
    }

    #[test]
    fn strip_xml_test() {
        assert_eq!(strip_xml_whitespace("<>    "), "<>".to_string());
        assert_eq!(strip_xml_whitespace("<>    <>  <>"), "<><><>".to_string());

        assert_eq!(
            strip_xml_whitespace(SVG_TO_STRIP),
            r###"<svg xmlns="http://www.w3.org/2000/svg" aria-label="i"><title>i </title><linearGradient id="s" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient></svg>"###
        )
    }
}
