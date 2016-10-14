develop:
	pip install --verbose --editable .

build:
	cargo build --release

.PHONY: build develop
