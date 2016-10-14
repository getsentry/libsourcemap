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
    }
}
