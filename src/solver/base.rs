use crate::expression::Expression;
use crate::number::Number;
use crate::number_theory::{factorial as fact, factorial_divide as fact_div};
use std::rc::Rc;

pub struct Limits {
    pub max_digits: usize,
    pub max_factorial: i128,
    pub max_quadratic_power: u8,
}

pub struct State<T: Number> {
    pub digits: usize,
    pub number: T,
    pub expression: Rc<Expression<T>>,
}

pub trait Solver<T: Number> {
    fn new(n: i128, limits: Limits) -> Self;

    fn n(&self) -> i128;

    fn get_max_digits(&self) -> usize;
    fn get_max_factorial_limit(&self) -> i128;

    fn solve(
        &mut self,
        target: i128,
        max_depth: Option<usize>,
    ) -> Option<(Rc<Expression<T>>, usize)>;
    fn search(&mut self, digits: usize) -> bool;

    fn unary_operation(&mut self, x: State<T>) -> bool {
        if self.n() == 1 || x.expression.get_divide().is_none() || !self.rational_check(x.number) {
            return false;
        }
        let number = x.number;
        let digits = x.digits;
        let (expression_numerator, expression_denominator) = x.expression.get_divide().unwrap();
        if is_single_digit(expression_denominator, self.n()) {
            return self.division_diff_one(
                number,
                digits,
                expression_numerator.clone(),
                expression_denominator.clone(),
            );
        }
        let mut lhs: &Rc<Expression<T>> = expression_denominator;
        let mut rhs: Option<Rc<Expression<T>>> = None;
        while let Some((p, q)) = lhs.get_multiply() {
            lhs = p;
            if is_single_digit(q, self.n()) {
                return self.division_diff_one(
                    number,
                    digits,
                    Expression::from_divide(
                        expression_numerator.clone(),
                        if let Some(r) = rhs.as_ref() {
                            Expression::from_multiply(lhs.clone(), r.clone())
                        } else {
                            lhs.clone()
                        },
                    ),
                    q.clone(),
                );
            }
            rhs = if let Some(r) = rhs {
                Some(Expression::from_multiply(q.clone(), r))
            } else {
                Some(q.clone())
            };
        }
        false
    }

    fn binary_operation(&mut self, x: State<T>, y: State<T>) -> bool {
        self.div(&x, &y)
            || self.mul(&x, &y)
            || self.add(&x, &y)
            || self.sub(&x, &y)
            || self.pow(&x, &y)
            || self.pow(&y, &x)
            || self.factorial_divide(&x, &y)
    }

    fn check(&mut self, x: T, digits: usize, expression: Rc<Expression<T>>) -> bool {
        if !self.range_check(x) || self.already_searched(x) {
            return false;
        }
        if self.insert(x, digits, expression.clone()) {
            return true;
        }
        let state = State {
            digits,
            number: x,
            expression: expression.clone(),
        };
        if self.sqrt(&state) {
            true
        } else if self.integer_check(x) {
            self.factorial(&state)
        } else {
            false
        }
    }

    fn range_check(&self, x: T) -> bool;
    fn integer_check(&self, x: T) -> bool;
    fn rational_check(&self, x: T) -> bool;

    fn already_searched(&self, x: T) -> bool;
    fn insert(&mut self, x: T, digits: usize, expression: Rc<Expression<T>>) -> bool;
    fn insert_extra(&mut self, x: T, depth: usize, digits: usize, expression: Rc<Expression<T>>);

    fn concat(&mut self, digits: usize) -> bool {
        if digits as f64 * 10f64.log2() - 9f64.log2() > self.get_max_digits() as f64 {
            return false;
        }
        let x = T::from_int((10i128.pow(digits as u32) - 1) / 9 * self.n());
        self.check(x, digits, Expression::from_number(x))
    }

    fn add(&mut self, x: &State<T>, y: &State<T>) -> bool;
    fn sub(&mut self, x: &State<T>, y: &State<T>) -> bool;
    fn mul(&mut self, x: &State<T>, y: &State<T>) -> bool;
    fn div(&mut self, x: &State<T>, y: &State<T>) -> bool;
    fn pow(&mut self, x: &State<T>, y: &State<T>) -> bool;
    fn sqrt(&mut self, x: &State<T>) -> bool;

    fn factorial(&mut self, x: &State<T>) -> bool {
        if let Some(n) = x.number.to_int() {
            if n < self.get_max_factorial_limit() as i128 {
                self.check(
                    T::from_int(fact(n)),
                    x.digits,
                    Expression::from_factorial(x.expression.clone()),
                )
            } else {
                false
            }
        } else {
            false
        }
    }

    fn factorial_divide(&mut self, x: &State<T>, y: &State<T>) -> bool {
        if x.number == y.number {
            return false;
        }
        let x_int = x.number.to_int();
        let y_int = y.number.to_int();
        if x_int.is_none() || y_int.is_none() {
            return false;
        }
        let mut x_int = x_int.unwrap();
        let mut y_int = y_int.unwrap();
        let mut x = x;
        let mut y = y;
        if x_int < y_int {
            let temp = x;
            x = y;
            y = temp;
            let temp = x_int;
            x_int = y_int;
            y_int = temp;
        }
        if x_int <= self.get_max_factorial_limit() as i128
            || y_int <= 2
            || x_int - y_int == 1
            || (x_int - y_int) as f64 * ((x_int as f64).log2() + (y_int as f64).log2())
                > self.get_max_digits() as f64 * 2.0
        {
            return false;
        }
        let x_expression = Expression::from_factorial(x.expression.clone());
        let y_expression = Expression::from_factorial(y.expression.clone());
        self.check(
            T::from_int(fact_div(x_int, y_int)),
            x.digits + y.digits,
            Expression::from_divide(x_expression.clone(), y_expression.clone()),
        )
    }

    fn division_diff_one(
        &mut self,
        x: T,
        digits: usize,
        numerator: Rc<Expression<T>>,
        denominator: Rc<Expression<T>>,
    ) -> bool;
}

fn is_single_digit<T: Number>(expression: &Expression<T>, n: i128) -> bool {
    match expression {
        Expression::Number(x) => x.to_int() == Some(n),
        Expression::Negate(x) => is_single_digit(x, n),
        Expression::Sqrt(x, _) => is_single_digit(x, n),
        Expression::Factorial(x) => is_single_digit(x, n),
        _ => false,
    }
}
