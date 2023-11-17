use cranelift::codegen::settings::{self, Configurable};
use cranelift::codegen::Context;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataDescription, DataId, FuncId, Linkage, Module};
use std::collections::HashMap;

fn println_u32(n: u32) {
    println!("{n}");
}

pub struct JITContext {
    pub(crate) module: JITModule,
    pub(crate) ctx: Context,
    pub(crate) func_ctx: FunctionBuilderContext,
    pub(crate) data_description: DataDescription,
    pub(crate) variables: HashMap<String, DataId>,
    pub(crate) print_func: FuncId,
    pub(crate) stmt_index: usize,
}

impl JITContext {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        // FIXME set back to true once the x64 backend supports it.
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let mut module = {
            let mut jit_builder = JITBuilder::with_isa(isa, default_libcall_names());
            let println_u32_addr: *const u8 = println_u32 as *const u8;
            jit_builder.symbol("println_u32", println_u32_addr);
            JITModule::new(jit_builder)
        };
        let mut sig_println_u32 = module.make_signature();
        sig_println_u32.params.push(AbiParam::new(types::I32));
        let func_println_u32 = module
            .declare_function("println_u32", Linkage::Import, &sig_println_u32)
            .unwrap();

        let ctx = module.make_context();
        let func_ctx = FunctionBuilderContext::new();
        let data_description = DataDescription::new();

        Self {
            module,
            ctx,
            func_ctx,
            data_description,
            variables: HashMap::new(),
            print_func: func_println_u32,
            stmt_index: 0,
        }
    }

    pub(crate) fn get_finalized_function(&mut self, func_id: FuncId) -> extern "C" fn() {
        // Perform linking.
        self.module.finalize_definitions().unwrap();

        let raw_func_ptr = self.module.get_finalized_function(func_id);
        // Cast it to a rust function pointer type.
        unsafe { std::mem::transmute::<_, extern "C" fn()>(raw_func_ptr) }
    }
}
