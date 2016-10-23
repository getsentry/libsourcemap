use std::io;
use std::str::Utf8Error;

use sourcemap;


error_chain! {
    foreign_links {
        io::Error, IoError;
        Utf8Error, Utf8Error;
        sourcemap::Error, SourceMapError;
    }

    errors {
        InternalError {
            description("Internal library error")
        }
        UnsupportedMemDbVersion {
            description("Unsupported memdb version")
        }
        BadMemDb {
            description("Bad memdb data")
        }
        TooManySources {
            description("Too many sources in the file for memdb")
        }
        TooManyNames {
            description("Too many names in the file for memdb")
        }
        LocationOverflow {
            description("File locations too large for memdb")
        }
        AlreadyMemDb {
            description("Cannot dump memdb from memdb view")
        }
    }
}
