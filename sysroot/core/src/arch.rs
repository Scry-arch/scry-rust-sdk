//! Architecture-specific functionality: the inline-assembly macros.
//! (Note: cg_clif cannot compile `asm!` for Scry, link hand-written assembly
//! as extern functions instead. These exist so the paths resolve.)

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro asm() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro global_asm() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro naked_asm() {
    /* compiler built-in */
}
