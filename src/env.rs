use crate::{
    expr::{Expression, Identifier},
    stmt::Statement,
    type_::{Type, TypeRepr},
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub enum Error {
    TypeError(TypeRepr, TypeRepr),
    VarNameError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError(left, right) => {
                write!(f, "Type error: not same types (`{left:?}` and `{right:?}`)",)
            }
            Self::VarNameError(name) => write!(f, "Name error: variable `{name}` not found"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env(Vec<Type>, HashMap<String, TypeRepr>);

impl Env {
    pub fn new(types: Vec<Type>, table: HashMap<String, TypeRepr>) -> Self {
        Self(types, table)
    }

    pub fn new_empty() -> Self {
        Self::new(Vec::new(), HashMap::new())
    }

    pub fn analyze_stmt(&mut self, stmt: &Statement) -> Result<(), Error> {
        match stmt {
            Statement::Assign(ident, expr) => {
                let ident_type = self.add_var(ident);
                let expr_type = self.analyze_expr(&expr)?;
                self.unify(&ident_type, &expr_type)?;
            }
            Statement::If(cond, body_stmt) => {
                let cond_type = self.analyze_expr(&cond)?;
                self.unify(&cond_type, &TypeRepr::Bool)?;
                self.analyze_stmt(&body_stmt)?;
            }
            Statement::Print(expr) => {
                let expr_type = self.analyze_expr(&expr)?;
                self.unify(&expr_type, &TypeRepr::Int)?;
            }
        }
        Ok(())
    }

    pub(self) fn analyze_expr(&mut self, expr: &Expression) -> Result<TypeRepr, Error> {
        match expr {
            Expression::Identifier(ident) => {
                let ident: &String = &*ident;
                if let Some(type_repr) = self.1.get(ident) {
                    Ok(*type_repr)
                } else {
                    Err(Error::VarNameError(ident.clone()))
                }
            }
            Expression::Add(left, right) | Expression::Sub(left, right) => {
                let left_type = self.analyze_expr(&left)?;
                self.unify(&left_type, &TypeRepr::Int)?;
                let right_type = self.analyze_expr(&right)?;
                self.unify(&right_type, &TypeRepr::Int)?;
                Ok(TypeRepr::Int)
            }
            Expression::Comp(left, right) => {
                let left_type = self.analyze_expr(&left)?;
                self.unify(&left_type, &TypeRepr::Int)?;
                let right_type = self.analyze_expr(&right)?;
                self.unify(&right_type, &TypeRepr::Int)?;
                Ok(TypeRepr::Bool)
            }
            Expression::Number(_) => Ok(TypeRepr::Int),
        }
    }

    fn add_var(&mut self, var_name: &Identifier) -> TypeRepr {
        let var_name: &String = &*var_name;
        if let Some(var_type) = self.1.get(var_name) {
            *var_type
        } else {
            let types_size = self.0.len();
            let type_repr = TypeRepr::Id(types_size);
            self.1.insert(var_name.clone(), type_repr);
            self.0.push(Type::Var(types_size, None));
            type_repr
        }
    }

    pub(self) fn unify(&mut self, left: &TypeRepr, right: &TypeRepr) -> Result<(), Error> {
        let left_resolved = self.resolve(left);
        let right_resolved = self.resolve(right);

        match (left_resolved, right_resolved) {
            // 左の型表現が型変数なら、
            (TypeRepr::Id(left_id), _) => {
                // 右の型表現が左と等しくない型変数、あるいは具体的な型である時に、左の型変数に型代入する
                if left_resolved != right_resolved {
                    if let Type::Var(id, _) = self.0.get(left_id).unwrap() {
                        *self.0.get_mut(left_id).unwrap() = Type::Var(*id, Some(right_resolved))
                    }
                }
                Ok(())
            }
            // 右の型表現が型変数の時は、左右を入れ替えて単一化
            (_, TypeRepr::Id(_)) => self.unify(right, left),
            // 左右の型表現とも具体的な型である場合は、左右が等しい場合に成功とする
            // 左右の型表現が等しくなければ型エラーとする
            (_, _) => {
                if left_resolved == right_resolved {
                    Ok(())
                } else {
                    Err(Error::TypeError(left_resolved, right_resolved))
                }
            }
        }
    }

    fn resolve(&mut self, ty: &TypeRepr) -> TypeRepr {
        let (id, repr) = match ty {
            TypeRepr::Id(id) => match self.0.get(*id).unwrap() {
                Type::Var(_, Some(t)) => (*id, *t),
                _ => return *ty,
            },
            _ => return *ty,
        };
        let resolved = self.resolve(&repr);
        *self.0.get_mut(id).unwrap() = Type::Var(id, Some(resolved));
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::Env;
    use crate::{env::Error, expr::*, stmt::*, type_::TypeRepr};

    #[test]
    fn type_equality_test() {
        let mut env = Env::new_empty();
        assert!(env.unify(&TypeRepr::Int, &TypeRepr::Int).is_ok());
        assert!(env.unify(&TypeRepr::Bool, &TypeRepr::Bool).is_ok());
    }

    fn add_var(env: &mut Env, var_name: Identifier, expr: Expression) {
        let assign_stmt = Statement::Assign(var_name.clone(), expr);
        assert!(env.analyze_stmt(&assign_stmt).is_ok());
        let ident_type = env.analyze_expr(&Expression::Identifier(var_name)).unwrap();
        assert!(env.unify(&ident_type, &TypeRepr::Int).is_ok());
    }

    #[test]
    fn var_type_test() {
        let mut env = Env::new_empty();
        add_var(
            &mut env,
            "a".to_owned().into(),
            Expression::Number(1u32.into()),
        );
    }

    #[test]
    fn int_expr_type_test() {
        let mut env = Env::new_empty();
        let expr = Expression::Add(
            Box::new(Expression::Number(1u32.into())),
            Box::new(Expression::Number(2u32.into())),
        );
        let expr_type = env.analyze_expr(&expr).unwrap();
        assert_eq!(expr_type, TypeRepr::Int);
    }

    #[test]
    fn var_and_int_expr_types_test() {
        let mut env = Env::new_empty();

        let ident_a: Identifier = "a".to_owned().into();
        add_var(&mut env, ident_a.clone(), Expression::Number(3u32.into()));

        let ident_b: Identifier = "b".to_owned().into();
        add_var(&mut env, ident_b.clone(), Expression::Number(2u32.into()));

        let expr = Expression::Sub(
            Box::new(Expression::Identifier(ident_a)),
            Box::new(Expression::Identifier(ident_b)),
        );

        let expr_type = env.analyze_expr(&expr);
        assert!(expr_type.is_ok(), "{}", expr_type.unwrap_err());
        let expr_type = expr_type.unwrap();
        assert_eq!(expr_type, TypeRepr::Int);
    }

    #[test]
    fn var_of_var_type_test() {
        let mut env = Env::new_empty();

        let ident_a: Identifier = "a".to_owned().into();
        add_var(&mut env, ident_a.clone(), Expression::Number(3u32.into()));

        let ident_b: Identifier = "b".to_owned().into();
        add_var(&mut env, ident_b.clone(), Expression::Identifier(ident_a));

        let ident_c: Identifier = "c".to_owned().into();
        add_var(&mut env, ident_c, Expression::Identifier(ident_b));
    }

    #[test]
    fn bool_expr_type_test() {
        let mut env = Env::new_empty();
        let expr = Expression::Comp(
            Box::new(Expression::Number(1u32.into())),
            Box::new(Expression::Number(2u32.into())),
        );
        let expr_type = env.analyze_expr(&expr).unwrap();
        assert_eq!(expr_type, TypeRepr::Bool);
    }

    #[test]
    fn var_and_bool_expr_types_test() {
        let mut env = Env::new_empty();

        let ident_a: Identifier = "a".to_owned().into();
        add_var(&mut env, ident_a.clone(), Expression::Number(3u32.into()));

        let ident_b: Identifier = "b".to_owned().into();
        add_var(&mut env, ident_b.clone(), Expression::Number(2u32.into()));

        let expr = Expression::Comp(
            Box::new(Expression::Identifier(ident_a)),
            Box::new(Expression::Identifier(ident_b)),
        );

        let expr_type = env.analyze_expr(&expr);
        assert!(expr_type.is_ok(), "{}", expr_type.unwrap_err());
        let expr_type = expr_type.unwrap();
        assert_eq!(expr_type, TypeRepr::Bool);
    }

    #[test]
    fn stmt_analyze_test() {
        let mut env = Env::new_empty();

        let ident_a: Identifier = "a".to_owned().into();
        add_var(&mut env, ident_a.clone(), Expression::Number(1u32.into()));

        let if_stmt = Statement::If(
            Expression::Comp(
                Box::new(Expression::Identifier(ident_a)),
                Box::new(Expression::Number(2u32.into())),
            ),
            Box::new(Statement::Print(Expression::Number(3u32.into()))),
        );

        assert!(env.analyze_stmt(&if_stmt).is_ok());
    }

    #[test]
    fn stmt_analyze_error_test() {
        let mut env = Env::new_empty();

        let if_stmt = Statement::If(
            Expression::Number(2u32.into()),
            Box::new(Statement::Print(Expression::Number(3u32.into()))),
        );

        assert!(matches!(
            env.analyze_stmt(&if_stmt),
            Err(Error::TypeError(_, _))
        ));
    }
}
