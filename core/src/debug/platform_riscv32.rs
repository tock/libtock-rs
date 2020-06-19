pub fn get_stack_pointer() -> usize {
    let stack_pointer;
    unsafe { llvm_asm!("mv $0, sp" : "=r"(stack_pointer) : : : "volatile") };
    stack_pointer
}
