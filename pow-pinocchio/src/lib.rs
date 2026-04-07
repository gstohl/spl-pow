#![cfg_attr(target_os = "solana", no_std)]

pub mod error;
pub mod hash;
pub mod instruction;
pub mod processor;
pub mod state;

pub use processor::process_instruction;

#[cfg(all(feature = "bpf-entrypoint", target_os = "solana"))]
mod entrypoint {
    use pinocchio::{no_allocator, nostd_panic_handler, program_entrypoint};

    use crate::process_instruction;

    program_entrypoint!(process_instruction);
    no_allocator!();
    nostd_panic_handler!();
}
