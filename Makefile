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

mac-wheels:
	SYMSYND_MACWHEELS=1 ./build-wheels.sh

manylinux-wheels:
	SYMSYND_MANYLINUX=1 ./docker-build.sh

all-wheels: mac-wheels manylinux-wheels

.PHONY: build develop wheel clean-docker mac-wheels manylinux-wheels all-wheels
