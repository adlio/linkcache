run:
	cargo build --features="bin" && \
	./target/debug/linkcache

workflow:
	mkdir -p target/workflow && \
	cargo build --features="bin" && \
	  cp ./target/debug/linkcache ./target/workflow
