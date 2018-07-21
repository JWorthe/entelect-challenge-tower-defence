build:
	cargo build --release

test:
	cargo test --release

bench:
	cargo run --release --features "benchmarking" --bin perf-test

profile:
	cargo build --release --features "benchmarking single-threaded"
	mkdir -p target/profile
	perf record -g target/release/perf-test
	perf script > target/profile/out.perf
	../FlameGraph/stackcollapse-perf.pl target/profile/out.perf > target/profile/out.folded
	../FlameGraph/flamegraph.pl target/profile/out.folded > target/profile/flamegraph.svg

clean:
	cargo clean

submission.zip: bot.json Cargo.lock Cargo.toml src
	zip -r9 submission.zip bot.json Cargo.lock Cargo.toml src

.PHONY: build test bench profile clean
