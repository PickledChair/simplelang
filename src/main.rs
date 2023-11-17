use simplelang::{codegen::Codegen, env::Env, jit_ctx::JITContext, parser};
use std::io::{self, Write};

fn main() {
    let mut jit_ctx = JITContext::new();
    let mut env = Env::new_empty();

    println!("If you want to quit, please enter `quit` or `exit`.");
    loop {
        let mut buffer = String::new();

        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line.");

        if ["quit", "exit"].contains(&buffer.trim()) {
            break;
        }
        if buffer.trim().is_empty() {
            continue;
        }

        match parser::parse(buffer.trim_start()) {
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
            Ok(stmt) => {
                if let Err(err) = env.analyze_stmt(&stmt) {
                    eprintln!("{err}");
                    continue;
                }

                let mut codegen = Codegen::new(&mut jit_ctx);
                let func_ptr = codegen.codegen_stmt(&stmt);
                // Call it!
                func_ptr();
            }
        }
    }
}
