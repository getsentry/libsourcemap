# Starter SConstruct for enscons

import sys
from distutils import sysconfig
import pytoml as toml
import enscons
import build

metadata = dict(toml.load(open('pyproject.toml')))['tool']['enscons']

# most specific binary, non-manylinux1 tag should be at the top of this list
import wheel.pep425tags
plat_tag = next(tag for tag in wheel.pep425tags.get_supported())[-1]
full_tag = '-'.join(('py2.py3', 'none', plat_tag))

env = Environment(tools=['default', 'packaging', enscons.generate],
                  PACKAGE_METADATA=metadata,
                  WHEEL_TAG=full_tag,
                  ROOT_IS_PURELIB=full_tag.endswith('-any'))

lib_path = 'libsourcemap/_libsourcemap.so'
rust_libname = 'liblibsourcemap' + env['SHLIBSUFFIX']
rust_lib = 'target/release/' + rust_libname

# Build rust
env.Command(
        target=rust_lib,
        source=["Cargo.toml"] + Glob("src/*.rs"),
        action="cargo build --release"
        )

# Copy compiled library into base directory
local_rust = env.Command(
        target=lib_path,
        source=rust_lib,
        action=Copy('$TARGET', '$SOURCE'))

# build cffi Python
def build_py(target, source, env):
    build.ffi.emit_python_code(str(target[0]))

cffi_py = env.Command(
    target='libsourcemap/_sourcemapnative.py',
    source=['build.py', 'include/libsourcemap.h'],
    action=build_py
)

# Add extra files or package_data here.
# cffi_py catches _sourcemapnative.py before it has been built.
# targets must be added to the wheel exactly once.
py_source = list(set(Glob('libsourcemap/*.py') + cffi_py))

platlib = env.Whl('platlib', py_source + [lib_path], root='')
whl = env.WhlFile(platlib)

# Add automatic source files, plus any other needed files.
sdist_source=FindSourceFiles() + ['PKG-INFO', 'setup.py']

sdist = env.SDist(source=sdist_source)

env.NoClean(sdist)
env.Alias('sdist', sdist)
