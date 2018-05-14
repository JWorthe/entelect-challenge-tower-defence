build:
	cargo build --release

test:
	cargo test --release

profile:
	cargo build --release --features "benchmarking"
	sudo perf record -F 1000 -a -g target/release/perf-test
	sudo perf script > out.perf
	../FlameGraph/stackcollapse-perf.pl out.perf > out.folded
	../FlameGraph/flamegraph.pl out.folded > flamegraph.svg

clean:
	cargo clean


.PHONY: build test profile clean
