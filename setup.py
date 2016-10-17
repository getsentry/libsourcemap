import os
import sys
import shutil
import subprocess

# The wheel generated carries a python unicode ABI tag.  We want to remove
# this since our wheel is actually universal as far as this goes since we
# never actually link against libpython.  Since there does not appear to
# be an API to do that, we just patch the internal function that wheel uses.
try:
    from wheel import pep425tags
except ImportError:
    pass
else:
    pep425tags.get_abi_tag = lambda: 'none'

from setuptools import setup, find_packages
from distutils.command.build_py import build_py
from distutils.command.build_ext import build_ext
from setuptools.dist import Distribution


# Build with clang if not otherwise specified.
if os.environ.get('LIBSOURCEMAP_MANYLINUX') == '1':
    os.environ.setdefault('CC', 'gcc')
    os.environ.setdefault('CXX', 'g++')
else:
    os.environ.setdefault('CC', 'clang')
    os.environ.setdefault('CXX', 'clang++')


PACKAGE = 'libsourcemap'
EXT_EXT = sys.platform == 'darwin' and '.dylib' or '.so'


def build_libsourcemap(base_path):
    lib_path = os.path.join(base_path, '_libsourcemap.so')
    here = os.path.abspath(os.path.dirname(__file__))
    cmdline = ['cargo', 'build', '--release']
    if not sys.stdout.isatty():
        cmdline.append('--color=always')
    rv = subprocess.Popen(cmdline, cwd=here).wait()
    if rv != 0:
        sys.exit(rv)
    src_path = os.path.join(here, 'target', 'release',
                            'liblibsourcemap' + EXT_EXT)
    if os.path.isfile(src_path):
        shutil.copy2(src_path, lib_path)


class CustomBuildPy(build_py):
    def run(self):
        build_py.run(self)
        build_libsourcemap(os.path.join(self.build_lib, *PACKAGE.split('.')))


class CustomBuildExt(build_ext):
    def run(self):
        build_ext.run(self)
        if self.inplace:
            build_py = self.get_finalized_command('build_py')
            build_libsourcemap(build_py.get_package_dir(PACKAGE))


class BinaryDistribution(Distribution):
    """This is necessary because otherwise the wheel does not know that
    we have non pure information.
    """
    def has_ext_modules(foo):
        return True


setup(
    name='libsourcemap',
    version='0.3.1',
    url='http://github.com/getsentry/libsourcemap',
    description='Helps working with sourcemaps.',
    license='BSD',
    author='Sentry',
    author_email='hello@getsentry.com',
    packages=find_packages(),
    cffi_modules=['build.py:ffi'],
    cmdclass={
        'build_ext': CustomBuildExt,
        'build_py': CustomBuildPy,
    },
    include_package_data=True,
    zip_safe=False,
    platforms='any',
    install_requires=[
        'cffi>=1.0.0',
    ],
    setup_requires=[
        'cffi>=1.0.0'
    ],
    classifiers=[
        'Intended Audience :: Developers',
        'License :: OSI Approved :: BSD License',
        'Operating System :: OS Independent',
        'Programming Language :: Python',
        'Topic :: Internet :: WWW/HTTP :: Dynamic Content',
        'Topic :: Software Development :: Libraries :: Python Modules'
    ],
    ext_modules=[],
    distclass=BinaryDistribution
)
