run:
	cargo build --features="bin" && \
	./target/debug/linkcache

release:
	cargo build --all-targets --all-features --release

coverage:
	cargo tarpaulin --all-targets --all-features --examples --exclude-files tests/* --out Html


build_workflow: release
	mkdir -p target/workflow && \
	cp ./target/release/linkcache ./target/workflow

test_workflow: build_workflow
	alfred_workflow_data=./test_workflow/workflow_data \
	alfred_workflow_cache=./test_workflow/workflow_cache \
	./target/workflow/linkcache test
