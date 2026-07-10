use pest::iterators::Pair;

use x_ast::Expression;
use x_ast::Statement;
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

type ExpressionBlock = (Vec<Statement>, Option<Box<Expression>>);
