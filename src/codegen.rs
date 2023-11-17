use crate::{expr::*, jit_ctx::JITContext, stmt::*};
use codegen::ir::UserFuncName;
use cranelift::prelude::*;
use cranelift_jit::JITModule;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use std::collections::HashMap;

pub struct Codegen<'a> {
    jit: &'a mut JITContext,
}

impl<'a> Codegen<'a> {
    pub fn new(jit: &'a mut JITContext) -> Self {
        Self { jit }
    }

    pub fn codegen_stmt(&mut self, stmt: &Statement) -> FuncId {
        let func_name = format!("stmt{}", self.jit.stmt_index);
        let func_sig = self.jit.module.make_signature();
        let func_id = self
            .jit
            .module
            .declare_function(&func_name, Linkage::Local, &func_sig)
            .unwrap();

        self.jit.ctx.func.signature = func_sig;
        self.jit.ctx.func.name = UserFuncName::user(0, func_id.as_u32());

        let func_builder: FunctionBuilder =
            FunctionBuilder::new(&mut self.jit.ctx.func, &mut self.jit.func_ctx);

        let mut stmt_codegen = StmtCodegen {
            module: &mut self.jit.module,
            func_builder,
            data_description: &mut self.jit.data_description,
            variables: &mut self.jit.variables,
            print_func: self.jit.print_func,
        };

        let block = stmt_codegen.func_builder.create_block();
        stmt_codegen.func_builder.switch_to_block(block);

        stmt_codegen.codegen_stmt(stmt);

        stmt_codegen.func_builder.ins().return_(&[]);
        stmt_codegen.func_builder.seal_all_blocks();
        stmt_codegen.func_builder.finalize();

        self.jit
            .module
            .define_function(func_id, &mut self.jit.ctx)
            .unwrap();
        self.jit.module.clear_context(&mut self.jit.ctx);

        self.jit.stmt_index += 1;
        func_id
    }
}

struct StmtCodegen<'a> {
    module: &'a mut JITModule,
    func_builder: FunctionBuilder<'a>,
    data_description: &'a mut DataDescription,
    variables: &'a mut HashMap<String, DataId>,
    print_func: FuncId,
}

impl<'a> StmtCodegen<'a> {
    pub fn codegen_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Print(expr) => self.codegen_print(expr),
            Statement::Assign(ident, expr) => {
                let ident_str: &str = &*ident;
                if self.variables.contains_key(ident_str) {
                    self.codegen_assign(ident, expr);
                } else {
                    self.codegen_def_var(ident, expr);
                }
            }
            Statement::If(cond, then) => self.codegen_if(cond, then),
        }
    }

    fn codegen_print(&mut self, expr: &Expression) {
        let local_func = self
            .module
            .declare_func_in_func(self.print_func, &mut self.func_builder.func);
        let arg = self.codegen_expr(expr);
        self.func_builder.ins().call(local_func, &[arg]);
    }

    fn codegen_assign(&mut self, ident: &Identifier, expr: &Expression) {
        let ident: &str = &*ident;
        let global_ref = {
            let data = *self.variables.get(ident).unwrap();
            let var = self
                .module
                .declare_data_in_func(data, &mut self.func_builder.func);
            self.func_builder
                .ins()
                .global_value(self.module.target_config().pointer_type(), var)
        };
        let value = self.codegen_expr(expr);
        self.func_builder
            .ins()
            .store(MemFlags::new(), value, global_ref, 0);
    }

    fn codegen_def_var(&mut self, ident: &Identifier, expr: &Expression) {
        let ident_str: &str = &*ident;
        let data = self
            .module
            .declare_data(ident_str, Linkage::Local, true, false)
            .unwrap();
        match expr {
            // NOTE: 整数値は以下のようにも初期化できるが、簡単のため処理を other の節に統一した
            // Expression::Number(num) => {
            //     let num: u32 = num.into();
            //     self.data_description.define(Box::new(num.to_ne_bytes()));
            //     self.module
            //         .define_data(data, self.data_description)
            //         .unwrap();
            //     self.variables.insert(ident_str.to_owned(), data);
            // }
            Expression::Comp(_, _) => unreachable!(),
            other => {
                self.data_description.define_zeroinit(4);
                self.module
                    .define_data(data, self.data_description)
                    .unwrap();
                self.variables.insert(ident_str.to_owned(), data);
                self.codegen_assign(ident, other);
            }
        }
        self.data_description.clear();
    }

    fn codegen_if(&mut self, cond: &Expression, then: &Statement) {
        let cond_val = self.codegen_expr(cond);

        let then_block = self.func_builder.create_block();
        let else_block = self.func_builder.create_block();
        let merge_block = self.func_builder.create_block();

        self.func_builder
            .ins()
            .brif(cond_val, then_block, &[], else_block, &[]);

        self.func_builder.switch_to_block(then_block);
        self.func_builder.seal_block(then_block);
        self.codegen_stmt(then);
        self.func_builder.ins().jump(merge_block, &[]);

        self.func_builder.switch_to_block(else_block);
        self.func_builder.seal_block(else_block);
        self.func_builder.ins().jump(merge_block, &[]);

        self.func_builder.switch_to_block(merge_block);
        self.func_builder.seal_block(merge_block);
    }

    fn codegen_expr(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Number(num) => {
                let num: u32 = num.into();
                self.func_builder.ins().iconst(types::I32, num as i64)
            }
            Expression::Add(lhs, rhs) => {
                let lhs = self.codegen_expr(lhs);
                let rhs = self.codegen_expr(rhs);
                self.func_builder.ins().iadd(lhs, rhs)
            }
            Expression::Sub(lhs, rhs) => {
                let lhs = self.codegen_expr(lhs);
                let rhs = self.codegen_expr(rhs);
                self.func_builder.ins().isub(lhs, rhs)
            }
            Expression::Comp(lhs, rhs) => {
                let lhs = self.codegen_expr(lhs);
                let rhs = self.codegen_expr(rhs);
                self.func_builder.ins().icmp(IntCC::Equal, lhs, rhs)
            }
            Expression::Identifier(ident) => {
                let ident: &str = &*ident;
                let global_ref = {
                    let data = *self.variables.get(ident).unwrap();
                    let global_var = self
                        .module
                        .declare_data_in_func(data, &mut self.func_builder.func);
                    self.func_builder
                        .ins()
                        .global_value(self.module.target_config().pointer_type(), global_var)
                };
                self.func_builder
                    .ins()
                    .load(types::I32, MemFlags::new(), global_ref, 0)
            }
        }
    }
}
