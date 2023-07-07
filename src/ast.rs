use std::borrow::Cow;
use anyhow::{anyhow, Result};
use pest::{Parser, iterators::Pair};

use crate::{Rule, Z80Parser};

pub fn parse(file_text: &str) -> Result<Program<'_>> {
  let mut program = Z80Parser::parse(Rule::program, file_text)?;
  let ast = program.next().and_then(build_ast);

  match ast {
      Some(Ast::Program(p)) => Ok(p),
      _ => Err(anyhow!("Failed to find program"))
  }
}

fn build_ast(pair: Pair<Rule>) -> Option<Ast> {
  match pair.as_rule() {
      Rule::program => {
          let statements : Vec<_> = pair.into_inner().filter_map(|pair| {
              if let Some(Ast::Statement( s )) = build_ast(pair) {
                  Some(s)
              } else {
                  None
              }
          }).collect();
          Some(Ast::Program(Program { statements }))
      },
      Rule::statement => {
          let mut label = None;
          let mut operation = None;
          let mut comment = None;

          for p in pair.into_inner() {
              match p.as_rule() {
                  Rule::label => label = Some(p.as_str()),
                  Rule::commentText => comment = Some(p.as_str()),
                  _ => {
                      let ast = build_ast(p);
                      if let Some(Ast::Operation( o )) = ast {
                          operation = Some(o);
                      }
                  }
              }
          }

          Some(Ast::Statement(Statement { label, comment, operation }))
      },
      Rule::operation => {
          let mut instruction = None;
          let mut parameters = vec![];

          for p in pair.into_inner() {
              match p.as_rule() {
                  Rule::instruction |
                  Rule::directive |
                  Rule::specialDirective => instruction = Some(p.as_str()),
                  Rule::expression => {
                      if let Some(Ast::Expression(e)) = build_ast(p) {
                          parameters.push(e);
                      }
                  }
                  _ => {},
              }
          }
          // TODO: don't panic
          Some(Ast::Operation(Operation { instruction: instruction.unwrap(), parameters}))
      },
      Rule::expression => {
          expr_to_string(pair).map(|v| Ast::Expression(Expression{ value: Cow::Borrowed(v) }))
      },
      _ => None
  }
}


fn expr_to_string(pair: Pair<Rule>) -> Option<&str> {
  let inner = pair.into_inner().next();
  if let Some(e) = inner {
      return match e.as_rule() {
          Rule::literal => expr_to_string(e),
          Rule::label => expr_to_string(e),
          Rule::register => expr_to_string(e),
          Rule::stringLiteral => expr_to_string(e),
          _ => Some(e.as_str())
      }
  }
  None
}

#[derive(Debug)]
enum Ast<'a> {
    Program(Program<'a>),
    Statement(Statement<'a>),
    Operation(Operation<'a>),
    Expression(Expression<'a>)
}

#[derive(Debug)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>
}

#[derive(Debug)]
pub struct Statement<'a>  {
    pub label: Option<&'a str>,
    pub comment: Option<&'a str>,
    pub operation: Option<Operation<'a>>,
}

impl<'a> Statement<'a> {
    pub fn operation_name(&'a self) -> Option<&'a str> {
        self.operation.as_ref().map(|o| o.instruction )
    }
}

#[derive(Debug)]
pub struct Operation<'a> {
    pub instruction: &'a str,
    pub parameters: Vec<Expression<'a>>,
}

impl<'a> Operation<'a> {
    pub fn first_param_as_str(&'a self) -> Option<&'a str>  {
        self.parameters.first().map(|p| p.value.as_ref())
    }
}

#[derive(Debug)]
pub struct Expression<'a> {
    pub value: Cow<'a, str>
}