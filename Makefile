run:
	cargo build --features="bin" && \
	./target/debug/linkcache

build_workflow:
	mkdir -p target/workflow && \
	cargo build --features="bin" && \
	cp ./target/debug/linkcache ./target/workflow

test_workflow: build_workflow
	alfred_workflow_data=./test_workflow/workflow_data \
	alfred_workflow_cache=./test_workflow/workflow_cache \
	./target/workflow/linkcache test

exp:
	jq '.items[] | { title, subtitle, arg }'