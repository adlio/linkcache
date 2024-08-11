run:
	cargo build --features="bin" && \
	./target/debug/linkcache test args three 4 five

