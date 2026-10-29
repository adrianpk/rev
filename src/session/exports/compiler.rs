use super::*;
use crate::org::export::compiler::{Compilation, Compiler};
impl Session {
    pub(super) fn confirm_compiler(
        &mut self,
        request: Request,
        prepared: Box<Prepared>,
    ) -> Result<(), String> {
        let compiler = Compiler::configured(self.buffers.snapshot())?;
        let label = format!("Run PDF compiler {}? yes/no: ", compiler.label());
        self.export_prompt(
            Prompt {
                request,
                backend: prepared.snapshot.request().backend,
                stage: Stage::Compiler(prepared, compiler),
            },
            &label,
        )
    }
    pub(super) fn compiler_argument(
        &mut self,
        request: Request,
        prepared: Box<Prepared>,
        compiler: Compiler,
        draft: &str,
    ) -> Result<(), String> {
        match draft {
            "yes" => {
                prepared.snapshot.buffers_current(&self.buffers)?;
                self.export_target_available(&prepared.target)?;
                self.launch_export(
                    request,
                    Work::Compile(Box::new(Compilation::new(*prepared, compiler))),
                )
            }
            "no" => {
                self.message("PDF compilation declined; destination unchanged");
                Ok(())
            }
            _ => Err("PDF compilation requires yes or no".into()),
        }
    }
}
