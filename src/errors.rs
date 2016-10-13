use std::io;

use sourcemap;


error_chain! {
    foreign_links {
        io::Error, IoError;
        sourcemap::Error, SourceMapError;
    }
}
