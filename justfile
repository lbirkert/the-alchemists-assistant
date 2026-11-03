run *FLAGS:
	cargo run -- {{FLAGS}}

# Might now work, see <https://github.com/rust-lang/rustc_codegen_cranelift/issues/1707>
run-cranelift *FLAGS:
	CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift cargo run -Zcodegen-backend -- {{FLAGS}}
