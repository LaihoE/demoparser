// Process-wide allocator. mimalloc cuts per-tick allocation cost and — most of all —
// cross-thread allocator contention in the multi-threaded second pass (the system allocator
// capped MT scaling). Measured: NaVi MT ~17% faster, test_demo MT ~24%. Output byte-identical.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(test)]
pub mod e2e_test;
pub mod first_pass;
pub mod maps;
pub mod parse_demo;
pub mod second_pass;
