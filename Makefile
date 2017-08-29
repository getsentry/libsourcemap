test: develop
	PYTHONPATH=. py.test --tb=short tests

develop:
	pip install --verbose --editable .

build:
	cargo build --release

sdist:
	python setup.py sdist --format=zip

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

build-all: sdist all-wheels

upload:
	@pip install twine
	@twine upload dist/libsourcemap-`python setup.py --version`[-.]*

release: build-all upload

.PHONY: test build develop wheel sdist clean-docker mac-wheels manylinux-wheels all-wheels build-all upload release
