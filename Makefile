build:
	cargo build --release

test:
	cargo test --release

profile:
	cargo build --release --features "benchmarking"
	mkdir -p target/profile
	sudo perf record -F 1000 -a -g target/release/perf-test
	sudo perf script > target/profile/out.perf
	../FlameGraph/stackcollapse-perf.pl target/profile/out.perf > target/profile/out.folded
	../FlameGraph/flamegraph.pl target/profile/out.folded > target/profile/flamegraph.svg

clean:
	cargo clean


.PHONY: build test profile clean
