use crate::ast::executable::{SelectionSet, TypeCondition};
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens, TryFromTokens, VariableDirectives};
use crate::lexical_token::PunctuatorType;
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct InlineFragment<'a> {
    type_condition: Option<TypeCondition<'a>>,
    directives: VariableDirectives<'a>,
    selection_set: SelectionSet<'a>,
    span: Span,
}

impl<'a> FromTokens<'a> for InlineFragment<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let ellipse_span = tokens.expect_punctuator(PunctuatorType::Ellipse)?;
        let type_condition = TypeCondition::try_from_tokens(tokens).transpose()?;
        let directives = VariableDirectives::from_tokens(tokens)?;
        let selection_set = SelectionSet::from_tokens(tokens)?;
        let span = ellipse_span.merge(selection_set.span());
        Ok(Self {
            type_condition,
            directives,
            selection_set,
            span,
        })
    }
}

impl<'a> IsMatch<'a> for InlineFragment<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::Ellipse)
            && tokens
                .peek_name(1)
                .map(|n| n.as_ref() == TypeCondition::ON)
                .unwrap_or(true)
    }
}

impl<'a> InlineFragment<'a> {
    pub fn type_condition(&self) -> Option<&TypeCondition<'a>> {
        self.type_condition.as_ref()
    }

    pub fn selection_set(&self) -> &SelectionSet<'a> {
        &self.selection_set
    }
}

impl<'a> bluejay_core::executable::InlineFragment for InlineFragment<'a> {
    type Directives = VariableDirectives<'a>;
    type SelectionSet = SelectionSet<'a>;

    fn type_condition(&self) -> Option<&str> {
        self.type_condition
            .as_ref()
            .map(|tc| tc.named_type().as_ref())
    }

    fn directives(&self) -> &Self::Directives {
        &self.directives
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}

impl<'a> HasSpan for InlineFragment<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}
