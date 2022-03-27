TARGET_ARCH := aarch64-unknown-linux-musl

build_dir:
	@mkdir -p build

clean:
	rm -rf build

services/%: build_dir
	.codebuild/make_dockerfile $*
	cargo build --target-dir target --release -p $$(.codebuild/service_pkgid $*)
	cp target/release/$* build

.PHONY: build_dir clean
