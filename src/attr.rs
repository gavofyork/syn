use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attribute {
    pub value: MetaItem,
    pub is_sugared_doc: bool,
}

/// A compile-time attribute item.
///
/// E.g. `#[test]`, `#[derive(..)]` or `#[feature = "foo"]`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MetaItem {
    /// Word meta item.
    ///
    /// E.g. `test` as in `#[test]`
    Word(Ident),
    /// List meta item.
    ///
    /// E.g. `derive(..)` as in `#[derive(..)]`
    List(Ident, Vec<MetaItem>),
    /// Name value meta item.
    ///
    /// E.g. `feature = "foo"` as in `#[feature = "foo"]`
    NameValue(Ident, String),
}

#[cfg(feature = "parsing")]
pub mod parsing {
    use super::*;
    use common::parsing::word;
    use helper::escaped_string;
    use nom::multispace;

    named!(pub attribute<&str, Attribute>, alt!(
        do_parse!(
            punct!("#") >>
            punct!("[") >>
            meta_item: meta_item >>
            punct!("]") >>
            (Attribute {
                value: meta_item,
                is_sugared_doc: false,
            })
        )
        |
        do_parse!(
            punct!("///") >>
            space: multispace >>
            content: take_until_s!("\n") >>
            (Attribute {
                value: MetaItem::NameValue(
                    "doc".into(),
                    format!("///{}{}", space, content),
                ),
                is_sugared_doc: true,
            })
        )
    ));

    named!(quoted<&str, String>, delimited!(
        punct!("\""),
        escaped_string,
        tag_s!("\"")
    ));

    named!(meta_item<&str, MetaItem>, alt!(
        do_parse!(
            ident: word >>
            punct!("(") >>
            inner: separated_list!(punct!(","), meta_item) >>
            punct!(")") >>
            (MetaItem::List(ident, inner))
        )
        |
        do_parse!(
            ident: word >>
            punct!("=") >>
            string: quoted >>
            (MetaItem::NameValue(ident, string))
        )
        |
        map!(word, MetaItem::Word)
    ));
}

#[cfg(feature = "printing")]
mod printing {
    use super::*;
    use quote::{Tokens, ToTokens};

    impl ToTokens for Attribute {
        fn to_tokens(&self, tokens: &mut Tokens) {
            tokens.append("#");
            tokens.append("[");
            self.value.to_tokens(tokens);
            tokens.append("]");
        }
    }

    impl ToTokens for MetaItem {
        fn to_tokens(&self, tokens: &mut Tokens) {
            match *self {
                MetaItem::Word(ref ident) => {
                    ident.to_tokens(tokens);
                }
                MetaItem::List(ref ident, ref inner) => {
                    ident.to_tokens(tokens);
                    tokens.append("(");
                    tokens.append_separated(inner, ",");
                    tokens.append(")");
                }
                MetaItem::NameValue(ref name, ref value) => {
                    name.to_tokens(tokens);
                    tokens.append("=");
                    value.to_tokens(tokens);
                }
            }
        }
    }
}