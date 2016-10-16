use std::io;

use sourcemap;


error_chain! {
    foreign_links {
        io::Error, IoError;
        sourcemap::Error, SourceMapError;
    }

    errors {
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
    }
}
