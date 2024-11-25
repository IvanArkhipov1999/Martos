extern crate alloc;

use alloc::boxed::Box;
use crate::task_manager::Task;

enum Architecture {
    Mips64,
    RiscV32,
    Xtensa,
}

fn create_switcher() -> Box<dyn SwitcherMethods> {
    #[cfg(target_arch = "mips64")]
    return Box::new(Mips64Switcher);

    #[cfg(target_arch = "riscv32")]
    return Box::new(RiscV32Switcher);

    #[cfg(target_arch = "xtensa")]
    return Box::new(XtensaSwitcher);

    #[cfg(not(any(target_arch = "mips64", target_arch = "riscv32", target_arch = "xtensa")))]
    return Box::new(Mips64Switcher);
}

pub(crate) struct ContextSwitcher {
    switcher: Box<dyn SwitcherMethods>,
}

impl ContextSwitcher {
    pub(crate) fn new() -> Self {
        ContextSwitcher {
            switcher: create_switcher(),
        }
    }
    pub(crate) fn save_context(&self, task: &Task) {
        self.switcher.save_context(task);
    }

    pub(crate) fn load_context(&self, task: &Task) {
        self.switcher.load_context(task);
    }

}

trait SwitcherMethods {
    fn save_context(task: &Task);
    fn load_context(task: &Task);
}

struct Mips64Switcher;
impl SwitcherMethods for Mips64Switcher {
    fn save_context(task: &Task) {}
    fn load_context(task: &Task) {}
}

struct RiscV32Switcher;
impl SwitcherMethods for RiscV32Switcher {
    fn save_context(task: &Task) {}
    fn load_context(task: &Task) {}
}

struct XtensaSwitcher;
impl SwitcherMethods for XtensaSwitcher {
    fn save_context(task: &Task) {}
    fn load_context(task: &Task) {}
}
