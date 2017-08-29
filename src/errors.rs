use std::io;
use std::str::Utf8Error;

use proguard;
use sourcemap;


error_chain! {
    foreign_links {
        Io(io::Error);
        Utf8(Utf8Error);
        SourceMap(sourcemap::Error);
        Proguard(proguard::Error);
    }

    errors {
        InternalError(msg: String) {
            description("Internal library error")
            display("Internal library error: {}", &msg)
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
