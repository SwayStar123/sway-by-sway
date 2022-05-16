use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Annotated<T: Parse> {
    pub attribute_list: Vec<AttributeDecl>,
    pub value: T,
}

macro_rules! impl_span_for_annotated (
    ($ty_name: ident) => {
        impl $ty_name {
            pub fn span(&self) -> Span {
                match self.attribute_list.first() {
                    Some(attr0) => Span::join(attr0.span(), self.value.span()),
                    None => self.value.span(),
                }
            }
        }
    };
);

pub(crate) use impl_span_for_annotated;

impl<T: Parse> Parse for Annotated<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let mut attribute_list = Vec::new();
        loop {
            if parser.peek::<HashToken>().is_some() {
                attribute_list.push(parser.parse()?);
            } else {
                break;
            }
        }
        let value = parser.parse()?;
        Ok(Annotated {
            attribute_list,
            value,
        })
    }
}

// Attributes can have any number of arguments:
//
//    #[attribute]
//    #[attribute()]
//    #[attribute(value)]
//    #[attribute(value0, value1, value2)]

#[derive(Clone, Debug)]
pub struct AttributeDecl {
    pub hash_token: HashToken,
    pub attribute: SquareBrackets<Attribute>,
}

impl AttributeDecl {
    pub fn span(&self) -> Span {
        Span::join(self.hash_token.span(), self.attribute.span())
    }
}

impl Parse for AttributeDecl {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let hash_token = parser.parse()?;
        let attribute = parser.parse()?;
        Ok(AttributeDecl {
            hash_token,
            attribute,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Parens<Punctuated<Ident, CommaToken>>>,
}

impl Attribute {
    pub fn span(&self) -> Span {
        self.args
            .as_ref()
            .map(|args| Span::join(self.name.span().clone(), args.span()))
            .unwrap_or_else(|| self.name.span().clone())
    }
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let name = parser.parse()?;
        let args = Parens::try_parse(parser)?;
        Ok(Attribute { name, args })
    }
}

impl ParseToEnd for Attribute {
    fn parse_to_end<'a, 'e>(mut parser: Parser<'a, 'e>) -> ParseResult<(Self, ParserConsumed<'a>)> {
        let attrib = parser.parse()?;
        match parser.check_empty() {
            Some(consumed) => Ok((attrib, consumed)),
            None => Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterAttribute)),
        }
    }
}
