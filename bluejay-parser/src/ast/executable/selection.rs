use crate::ast::executable::{Field, FragmentSpread, InlineFragment};
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::executable::{
    AbstractSelection, Selection as CoreSelection, SelectionFromAbstract,
};

#[derive(Debug)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

impl<'a> AbstractSelection for Selection<'a> {
    type Field = Field<'a>;
    type FragmentSpread = FragmentSpread<'a>;
    type InlineFragment = InlineFragment<'a>;

    fn as_ref(&self) -> SelectionFromAbstract<'_, Self> {
        match self {
            Self::Field(f) => CoreSelection::Field(f),
            Self::FragmentSpread(fs) => CoreSelection::FragmentSpread(fs),
            Self::InlineFragment(i) => CoreSelection::InlineFragment(i),
        }
    }
}

impl<'a> FromTokens<'a> for Selection<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if Field::is_match(tokens) {
            Field::from_tokens(tokens).map(Self::Field)
        } else if FragmentSpread::is_match(tokens) {
            FragmentSpread::from_tokens(tokens).map(Self::FragmentSpread)
        } else if InlineFragment::is_match(tokens) {
            InlineFragment::from_tokens(tokens).map(Self::InlineFragment)
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for Selection<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        Field::is_match(tokens) || tokens.peek_punctuator_matches(0, PunctuatorType::Ellipse)
    }
}
