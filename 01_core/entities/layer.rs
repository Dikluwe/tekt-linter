//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/violation-types.md
//! @prompt-hash 61cde08c
//! @layer L1
//! @updated 2026-03-13

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layer {
    L0,
    L1,
    L2,
    L3,
    L4,
    Lab,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_clone_and_eq() {
        let a = Layer::L1;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn layer_unknown_is_distinct() {
        assert_ne!(Layer::Unknown, Layer::L1);
        assert_ne!(Layer::Unknown, Layer::L2);
        assert_ne!(Layer::Unknown, Layer::L3);
        assert_ne!(Layer::Unknown, Layer::L4);
        assert_ne!(Layer::Unknown, Layer::Lab);
    }

    #[test]
    fn all_layers_debug() {
        let layers = [
            Layer::L0,
            Layer::L1,
            Layer::L2,
            Layer::L3,
            Layer::L4,
            Layer::Lab,
            Layer::Unknown,
        ];
        for layer in &layers {
            let s = format!("{:?}", layer);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn language_clone_and_eq() {
        let a = Language::Rust;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn language_unknown_is_distinct() {
        assert_ne!(Language::Unknown, Language::Rust);
        assert_ne!(Language::Unknown, Language::TypeScript);
        assert_ne!(Language::Unknown, Language::Python);
    }

    #[test]
    fn all_languages_debug() {
        let langs = [
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::Unknown,
        ];
        for lang in &langs {
            let s = format!("{:?}", lang);
            assert!(!s.is_empty());
        }
    }
}
