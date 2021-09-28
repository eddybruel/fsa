use {
    crate::nfa::{Builder, Nfa},
    std::{iter::Peekable, str::Chars},
};

#[derive(Debug, Default)]
pub struct Parser {
    nfa: Builder,
    operator_stack: Vec<Operator>,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, pattern: &str, token: usize) {
        Parse {
            builder: &mut self.nfa,
            operator_stack: &mut self.operator_stack,
            chars: pattern.chars().peekable(),
            token,
        }
        .parse()
    }

    pub fn build(self) -> Nfa {
        self.nfa.build()
    }
}

struct Parse<'a> {
    builder: &'a mut Builder,
    operator_stack: &'a mut Vec<Operator>,
    chars: Peekable<Chars<'a>>,
    token: usize,
}

impl<'a> Parse<'a> {
    fn parse(&mut self) {
        while let Some(ch) = self.chars.next() {
            match ch {
                '(' => self.operator_stack.push(Operator::LeftParenthesis),
                ')' => {
                    while let Some(operator) = self.operator_stack.pop() {
                        if operator.is_left_parenthesis() {
                            break;
                        }
                        self.apply_operator(operator);
                    }
                    self.try_concatenate();
                }
                '*' => {
                    self.builder.zero_or_more();
                    self.try_concatenate();
                }
                '+' => {
                    self.builder.one_or_more();
                    self.try_concatenate();
                }
                '?' => {
                    self.builder.zero_or_one();
                    self.try_concatenate();
                }
                '\\' => match self.chars.next().unwrap() {
                    '(' | ')' | '*' | '+' | '?' | '\\' | '|' => {
                        self.builder.char(ch);
                        self.try_concatenate();
                    }
                    _ => panic!(),
                },
                '|' => self.handle_operator(Operator::Alternate),
                ch => {
                    self.builder.char(ch);
                    self.try_concatenate();
                }
            }
        }
        while let Some(operator) = self.operator_stack.pop() {
            assert!(!operator.is_left_parenthesis());
            self.apply_operator(operator);
        }
        self.builder.accept(self.token);
    }

    fn try_concatenate(&mut self) {
        if self.chars.peek().map_or(false, |ch| !")*+?|".contains(*ch)) {
            self.handle_operator(Operator::Concatenate);
        }
    }

    fn handle_operator(&mut self, operator_0: Operator) {
        while let Some(operator_1) = self.operator_stack.last() {
            if operator_1.is_left_parenthesis() || !operator_1.groups_left(operator_0) {
                break;
            }
            let operator_1 = self.operator_stack.pop().unwrap();
            self.apply_operator(operator_1);
        }
        self.operator_stack.push(operator_0);
    }

    fn apply_operator(&mut self, operator: Operator) {
        match operator {
            Operator::Alternate => self.builder.alternate(),
            Operator::Concatenate => self.builder.concatenate(),
            Operator::LeftParenthesis => panic!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    LeftParenthesis,
    Alternate,
    Concatenate,
}

impl Operator {
    fn is_left_parenthesis(self) -> bool {
        match self {
            Self::LeftParenthesis => true,
            _ => false,
        }
    }

    fn groups_left(self, other: Self) -> bool {
        self.precedence() >= other.precedence()
    }

    fn precedence(self) -> u32 {
        match self {
            Operator::Alternate => 0,
            Operator::Concatenate => 1,
            Operator::LeftParenthesis => panic!(),
        }
    }
}
