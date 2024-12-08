use pest::Parser;
use pest_derive::Parser;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use regex::Regex;
use crate::error::{Error, Result};
use crate::types::{KeyPart, TagName};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GlobParser;

#[derive(Debug,Hash, Eq, PartialEq)]
pub struct FileKey(pub BTreeMap<KeyPart, String>);

impl FileKey {
    pub fn as_string(&self) -> String {
        let key = self.0
            .values()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("_");

        key
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub regex: Regex,
    pub placeholders: Vec<KeyPart>,
}

impl Pattern {
    pub fn parse(&self, path: &Path) -> Option<FileKey> {
        let path_str = path.to_string_lossy();
        let captures = self.regex.captures(&path_str)?;

        let mut parts = BTreeMap::new();
        for key_part in &self.placeholders {
            if let Some(value) = captures.name(&key_part.0) {
                parts.insert(key_part.clone(), value.as_str().to_owned());
            }
        }

        Some(FileKey(parts))
    }
}

#[derive(Debug)]
pub struct Declaration(pub HashMap<TagName, Pattern>);

pub fn parse_declaration(input: &str) -> Result<Declaration> {
    let pairs = GlobParser::parse(Rule::declaration, input)?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::declaration => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::anonymous_pattern => {
                        let pattern = HashMap::from([(TagName("main".to_string()), parse_single_pattern(
                            inner.into_inner().next().unwrap(),
                        )?)]);
                        return Ok(Declaration(pattern));
                    }
                    Rule::named_patterns => {
                        let mut patterns = HashMap::new();
                        for named_pattern in inner.into_inner() {
                            if named_pattern.as_rule() == Rule::named_pattern {
                                let mut parts = named_pattern.into_inner();
                                let name = parts.next().unwrap().as_str().to_string();
                                let pattern = parse_single_pattern(
                                    parts.next().unwrap(),
                                )?;
                                patterns.insert(TagName(name), pattern);
                            }
                        }
                        return Ok(Declaration(patterns));
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    unreachable!()
}

fn parse_single_pattern(pattern_pair: pest::iterators::Pair<Rule>) -> Result<Pattern> {
    assert_eq!(pattern_pair.as_rule(), Rule::pattern);
    let mut regex = String::new();
    let mut placeholders = Vec::new();
    for path_segment in pattern_pair.into_inner() {
        if !regex.is_empty() {
            regex.push('/');
        }
        let segment = path_segment.into_inner().next().unwrap();
        match segment.as_rule() {
            Rule::double_wildcard => {
                // not entirely sure of this
                regex.push_str("(?:(?:[^/]+/)*[^/]+)?");
            }
            Rule::element => {
                let mut element_regex = String::new();
                for part in segment.into_inner() {
                    for item in part.into_inner() {
                        match item.as_rule() {
                            Rule::literal => {
                                element_regex.push_str(&regex::escape(item.as_str()));
                            }
                            Rule::wildcard => {
                                element_regex.push_str("[^/]*");
                            }
                            Rule::option_group => {
                                element_regex.push('(');
                                for (i, alt) in item.into_inner().enumerate() {
                                    if i > 0 {
                                        element_regex.push('|');
                                    }
                                    element_regex.push_str(&regex::escape(alt.as_str()));
                                }
                                element_regex.push(')');
                            }
                            Rule::placeholder => {
                                let mut inner = item.into_inner();
                                let name = inner.next().unwrap().as_str();
                                placeholders.push(KeyPart(name.to_string().clone()));

                                let mut pattern = String::from("[^/]");
                                let mut quantifier = String::from("*");

                                if let Some(flags) = inner.next() {
                                    for flag in flags.into_inner() {
                                        match flag.as_rule() {
                                            Rule::digit_flag => {
                                                pattern = String::from("\\d");
                                            }
                                            Rule::alpha_flag => {
                                                pattern = String::from("[a-zA-Z0-9]");
                                            }
                                            Rule::greedy_flag => {
                                                quantifier = String::from("+");
                                            }
                                            Rule::exact_length => {
                                                quantifier = format!("{{{}}}", flag.as_str());
                                            }
                                            Rule::min_length => {
                                                let len = flag.as_str()[1..]
                                                    .parse::<usize>()
                                                    .map_err(|_| Error::InvalidPattern("Invalid minimum length".to_string()))?;
                                                quantifier = format!("{{{},}}", len);
                                            }
                                            Rule::max_length => {
                                                let len = flag.as_str()[1..]
                                                    .parse::<usize>()
                                                    .map_err(|_| Error::InvalidPattern("Invalid minimum length".to_string()))?;
                                                quantifier = format!("{{1,{}}}", len);
                                            }
                                            Rule::range_length => {
                                                let range: Vec<&str> = flag.as_str().split('-').collect();
                                                quantifier = format!("{{{},{}}}", range[0], range[1]);
                                            }
                                            _ => {}
                                        }
                                    }
                                }

                                element_regex.push_str("(?P<");
                                element_regex.push_str(name);
                                element_regex.push('>');
                                element_regex.push_str(&pattern);
                                element_regex.push_str(&quantifier);
                                element_regex.push(')');
                            }
                            _ => {}
                        }
                    }
                }
                regex.push_str(&element_regex);
            }
            _ => unreachable!(),
        }
    }

    // TODO: uncomment this only when the pattern starts with a /. Maybe worth including in the grammar?
    // regex.insert(0, '^');
    regex.push('$');
    let compiled_regex = Regex::new(&regex)
        .map_err(|e| Error::InvalidPattern(e.to_string()))?;
    Ok(Pattern { regex: compiled_regex, placeholders })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymous_pattern() -> Result<()> {
        let input = "foo/{name:a>3}/test.{ext:3}";
        let declaration = parse_declaration(input)?;
        assert!(declaration.0.contains_key(&TagName("main".to_string())));
        Ok(())
    }

    #[test]
    fn test_named_patterns() -> Result<()> {
        let input = "source=src/**/*.rs;test=tests/{name}_test.rs";
        let declaration = parse_declaration(input)?;
        assert!(declaration.0.contains_key(&TagName("source".to_string())));
        assert!(declaration.0.contains_key(&TagName("test".to_string())));
        Ok(())
    }
}
