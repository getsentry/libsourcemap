develop:
	pip install --verbose --editable .

build:
	cargo build --release

wheel:
	SYMSYND_MANYLINUX="$(MANYLINUX)" ./build-wheels.sh

clean-docker:
	docker rmi -f libsourcemap:dev
	docker rmi -f libsourcemap:dev32
	docker rmi -f libsourcemap:dev64

manylinux-wheels:
	SYMSYND_MANYLINUX=1 ./docker-build.sh

.PHONY: build develop wheel clean-docker manylinux-wheels
