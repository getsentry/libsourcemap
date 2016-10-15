test: develop
	PYTHONPATH=. py.test --tb=short tests

develop:
	pip install --verbose --editable .

build:
	cargo build --release

sdist:
	python setup.py sdist --formats=zip

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

release: sdist all-wheels
	pip install twine
	twine upload dist/libsourcemap-`python setup.py --version`[-.]*

.PHONY: test build develop wheel sdist clean-docker mac-wheels manylinux-wheels all-wheels release
