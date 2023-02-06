use std::fmt;

use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{char, multispace0},
    combinator::{eof, map, opt, value},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, terminated},
    Finish, IResult,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[derive(Debug, PartialEq)]
struct Recipe<'a> {
    base: &'a str,
    instructions: Vec<Instruction<'a>>,
}

#[derive(Debug, PartialEq)]
enum Instruction<'a> {
    AddIngredients { recipe: Recipe<'a>, optional: bool },
    Process(&'a str),
}

impl<'a> fmt::Display for Recipe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base)?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            if i == 0 {
                write!(f, "　に")?;
            }
            write!(f, "\n{}　をして", instruction)?;
        }
        write!(f, "\n完成！")
    }
}

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::AddIngredients { recipe, optional } => {
                if *optional {
                    write!(f, "お好みで　")?;
                }
                write!(f, "「{}　を", recipe.base)?;
                for i in &recipe.instructions {
                    write!(f, "　{}　して", i)?;
                }
                write!(f, "加える」")
            }
            Instruction::Process(s) => write!(f, "「{}」", s),
        }
    }
}

fn recipe(input: &str) -> IResult<&str, Recipe> {
    map(
        pair(
            preceded(skip, is_not(">)\r\n#")),
            opt(preceded(
                preceded(skip, char('>')),
                separated_list1(preceded(skip, char('>')), instruction),
            )),
        ),
        |(base, instructions)| Recipe {
            base: base.trim(),
            instructions: instructions.unwrap_or_default(),
        },
    )(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    alt((
        map(
            pair(
                opt(preceded(skip, char('?'))),
                preceded(
                    preceded(skip, char('+')),
                    alt((
                        delimited(preceded(skip, char('(')), recipe, preceded(skip, char(')'))),
                        map(preceded(skip, is_not(">\r\n#")), |s: &str| Recipe {
                            base: s.trim(),
                            instructions: Vec::new(),
                        }),
                    )),
                ),
            ),
            |(opt, recipe)| Instruction::AddIngredients {
                recipe,
                optional: opt.is_some(),
            },
        ),
        map(preceded(skip, is_not(">)\r\n#")), |s: &str| {
            Instruction::Process(s.trim())
        }),
    ))(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(
        char('#'),
        map(opt(is_not("\r\n")), Option::unwrap_or_default),
    )(input)
}

fn skip(input: &str) -> IResult<&str, ()> {
    delimited(multispace0, value((), opt(comment)), multispace0)(input)
}

fn parse(input: &str) -> IResult<&str, Recipe> {
    terminated(delimited(skip, recipe, skip), eof)(input)
}

#[wasm_bindgen]
pub fn transpile(input: &str) -> Result<JsValue, JsValue> {
    let (_, recipe) = parse(input)
        .finish()
        .map_err(|err| format!("parse error: {err}"))?;
    let s = recipe.to_string();
    Ok(JsValue::from_str(&s))
}

#[cfg(test)]
mod tests {
    use super::{parse, Instruction, Recipe};

    #[test]
    fn test() {
        assert_eq!(
            parse(
                r#"  # コメント
aaa > bbb # コメント
> + (#
    ccc>ddd>+( eee )
# コメント
) > ? + (
    fff>ggg
)
# コメント"#
            ),
            Ok((
                "",
                Recipe {
                    base: "aaa",
                    instructions: vec![
                        Instruction::Process("bbb"),
                        Instruction::AddIngredients {
                            recipe: Recipe {
                                base: "ccc",
                                instructions: vec![
                                    Instruction::Process("ddd"),
                                    Instruction::AddIngredients {
                                        recipe: Recipe {
                                            base: "eee",
                                            instructions: vec![]
                                        },
                                        optional: false
                                    }
                                ]
                            },
                            optional: false
                        },
                        Instruction::AddIngredients {
                            recipe: Recipe {
                                base: "fff",
                                instructions: vec![Instruction::Process("ggg")]
                            },
                            optional: true
                        }
                    ],
                }
            ))
        );
    }
}
