test:
	@cargo nextest run --all-features

cov:
	@cargo llvm-cov nextest --all-features --workspace --lcov --output-path coverage/lcov-$(shell date +%F).info

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -m "Update CHANGELOG.md" || true
	@cargo release push --execute

.PHONY: proto-build cov test
