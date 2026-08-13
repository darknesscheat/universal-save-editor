use std::collections::HashMap;

/// Text that a plugin may supply in more than one language.
///
/// The plain field is the fallback and is always present; `translations` maps a
/// locale tag to a replacement. A plugin author who only speaks one language
/// writes nothing extra and everything still works.
pub type Translations = HashMap<String, String>;

/// Pick the best translation for `locale`, falling back sensibly.
///
/// Matching goes from most to least specific:
/// 1. exact tag, `pt-BR`
/// 2. base language, `pt-BR` also accepts a `pt` entry
/// 3. any regional variant of the base: a `pt` request accepts `pt-BR`
/// 4. the untranslated default
///
/// Tags are compared case-insensitively, so `pt-br` and `pt-BR` are the same.
pub fn pick<'a>(default: &'a str, translations: &'a Translations, locale: &str) -> &'a str {
    if translations.is_empty() || locale.is_empty() {
        return default;
    }

    let wanted = locale.to_ascii_lowercase();

    if let Some(hit) = find(translations, |k| k == wanted) {
        return hit;
    }

    let base = base_language(&wanted);
    if base != wanted {
        if let Some(hit) = find(translations, |k| k == base) {
            return hit;
        }
    }

    if let Some(hit) = find(translations, |k| base_language(k) == base) {
        return hit;
    }

    default
}

fn find(translations: &Translations, matches: impl Fn(&str) -> bool) -> Option<&str> {
    // Sorted so the choice between two equally good candidates is stable
    // rather than dependent on hash order.
    let mut keys: Vec<&String> = translations.keys().collect();
    keys.sort();
    keys.into_iter()
        .find(|k| matches(&k.to_ascii_lowercase()))
        .map(|k| translations[k].as_str())
        .filter(|s| !s.is_empty())
}

fn base_language(tag: &str) -> String {
    tag.split(['-', '_'])
        .next()
        .unwrap_or(tag)
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> Translations {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn falls_back_to_the_default_when_there_are_no_translations() {
        assert_eq!(pick("Money", &Translations::new(), "tr"), "Money");
    }

    #[test]
    fn falls_back_to_the_default_for_an_unknown_locale() {
        assert_eq!(pick("Money", &map(&[("tr", "Para")]), "ja"), "Money");
    }

    #[test]
    fn takes_an_exact_match() {
        assert_eq!(
            pick("Money", &map(&[("tr", "Para"), ("de", "Geld")]), "tr"),
            "Para"
        );
    }

    #[test]
    fn matches_case_insensitively() {
        assert_eq!(
            pick("Money", &map(&[("pt-BR", "Dinheiro")]), "pt-br"),
            "Dinheiro"
        );
    }

    #[test]
    fn a_regional_request_accepts_a_base_language_entry() {
        assert_eq!(
            pick("Money", &map(&[("pt", "Dinheiro")]), "pt-BR"),
            "Dinheiro"
        );
    }

    #[test]
    fn a_base_request_accepts_a_regional_entry() {
        assert_eq!(
            pick("Money", &map(&[("pt-BR", "Dinheiro")]), "pt"),
            "Dinheiro"
        );
    }

    #[test]
    fn prefers_the_exact_tag_over_a_regional_variant() {
        let t = map(&[("pt", "Dinheiro PT"), ("pt-BR", "Dinheiro BR")]);
        assert_eq!(pick("Money", &t, "pt-BR"), "Dinheiro BR");
        assert_eq!(pick("Money", &t, "pt"), "Dinheiro PT");
    }

    #[test]
    fn underscore_tags_work_too() {
        assert_eq!(pick("Money", &map(&[("zh-CN", "金钱")]), "zh_CN"), "金钱");
    }

    #[test]
    fn an_empty_translation_does_not_blank_the_label() {
        assert_eq!(pick("Money", &map(&[("tr", "")]), "tr"), "Money");
    }

    #[test]
    fn an_empty_locale_uses_the_default() {
        assert_eq!(pick("Money", &map(&[("tr", "Para")]), ""), "Money");
    }
}
