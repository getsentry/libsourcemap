use std::ptr;
use std::mem;
use std::slice;
use std::panic;
use std::ffi::CStr;
use std::borrow::Cow;
use std::os::raw::{c_int, c_uint, c_char};

use sourcemap::Error as SourceMapError;
use errors::{Error, ErrorKind, Result};
use unified::{View, TokenMatch, Index, ViewOrIndex};
use memdb::DumpOptions;


fn resultbox<T>(val: T) -> Result<*mut T> {
    Ok(Box::into_raw(Box::new(val)))
}


#[derive(Debug)]
#[repr(C)]
pub struct Token {
    pub dst_line: c_uint,
    pub dst_col: c_uint,
    pub src_line: c_uint,
    pub src_col: c_uint,
    pub name: *const u8,
    pub name_len: c_uint,
    pub src: *const u8,
    pub src_len: c_uint,
    pub src_id: c_uint,
}


#[derive(Debug)]
#[repr(C)]
pub struct CError {
    pub message: *const u8,
    pub failed: c_int,
    pub code: c_int,
}

fn get_error_code_from_kind(kind: &ErrorKind) -> c_int {
    match *kind {
        ErrorKind::SourceMapError(SourceMapError::IndexedSourcemap) => 2,
        ErrorKind::SourceMapError(SourceMapError::BadJson(_, _, _)) => 3,
        ErrorKind::SourceMapError(SourceMapError::CannotFlatten(_)) => 4,
        ErrorKind::UnsupportedMemDbVersion => 5,
        ErrorKind::IoError(_) => 6,
        ErrorKind::TooManySources => 20,
        ErrorKind::TooManyNames => 21,
        ErrorKind::LocationOverflow => 22,
        ErrorKind::AlreadyMemDb => 23,
        _ => 1,
    }
}

unsafe fn set_token<'a>(out: *mut Token, tm: &'a TokenMatch<'a>) {
    (*out).dst_line = tm.dst_line;
    (*out).dst_col = tm.dst_col;
    (*out).src_line = tm.src_line;
    (*out).src_col = tm.src_col;
    (*out).name = match tm.name {
        Some(name) => name.as_ptr(),
        None => ptr::null()
    };
    (*out).name_len = tm.name.map(|x| x.as_bytes().len()).unwrap_or(0) as c_uint;
    (*out).src = match tm.src {
        Some(src) => src.as_ptr(),
        None => ptr::null()
    };
    (*out).src_len = tm.src.map(|x| x.as_bytes().len()).unwrap_or(0) as c_uint;
    (*out).src_id = tm.src_id;
}


unsafe fn notify_err(err: Error, err_out: *mut CError) {
    if !err_out.is_null() {
        let s = format!("{}\x00", err);
        (*err_out).failed = 1;
        (*err_out).message = Box::into_raw(s.into_boxed_str()) as *mut u8;
        (*err_out).code = get_error_code_from_kind(err.kind());
    }
}

unsafe fn landingpad<F: FnOnce() -> Result<T> + panic::UnwindSafe, T>(
    f: F, err_out: *mut CError) -> T
{
    if let Ok(rv) = panic::catch_unwind(f) {
        rv.map_err(|err| notify_err(err, err_out)).unwrap_or(mem::zeroed())
    } else {
        notify_err(ErrorKind::InternalError.into(), err_out);
        mem::zeroed()
    }
}

macro_rules! export (
    (
        $name:ident($($aname:ident: $aty:ty),*) -> Result<$rv:ty> $body:block
    ) => (
        #[no_mangle]
        pub unsafe extern "C" fn $name($($aname: $aty,)* err_out: *mut CError) -> $rv
        {
            landingpad(|| $body, err_out)
        }
    );
    (
        $name:ident($($aname:ident: $aty:ty),*) $body:block
    ) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name($($aname: $aty,)*)
        {
            // this silences panics and stuff
            landingpad(|| { $body; Ok(0 as c_int)}, ptr::null_mut());
        }
    }
);

export!(lsm_init() {
    fn silent_panic_handler(_pi: &panic::PanicInfo) {
        // don't do anything here.  This disables the default printing of
        // panics to stderr which we really don't care about here.
    }
    panic::set_hook(Box::new(silent_panic_handler));
});

export!(lsm_view_from_json(bytes: *const u8, len: c_uint) -> Result<*mut View> {
    resultbox(View::json_from_slice(slice::from_raw_parts(bytes, len as usize))?)
});

export!(lsm_view_from_memdb(
    bytes: *const u8, len: c_uint) -> Result<*mut View>
{
    // XXX: this currently copies because that's safer.  Consider improving this?
    resultbox(View::memdb_from_vec(slice::from_raw_parts(
        bytes,
        len as usize
    ).to_vec())?)
});

export!(lsm_view_from_memdb_file(path: *const c_char) -> Result<*mut View> {
    resultbox(View::memdb_from_path(CStr::from_ptr(path).to_str()?)?)
});

export!(lsm_view_free(view: *mut View) {
    if !view.is_null() {
        Box::from_raw(view);
    }
});

export!(lsm_view_get_token_count(view: *const View) -> Result<c_uint> {
    Ok((*view).get_token_count() as c_uint)
});

export!(lsm_view_get_token(view: *const View, idx: c_uint, out: *mut Token) -> Result<c_int> {
    Ok(match (*view).get_token(idx as u32) {
        None => 0,
        Some(tm) => {
            set_token(out, &tm);
            1
        }
    })
});

export!(lsm_view_lookup_token(
        view: *const View, line: c_uint, col: c_uint, out: *mut Token) -> Result<c_int>
{
    Ok(match (*view).lookup_token(line, col) {
        None => 0,
        Some(tm) => {
            set_token(out, &tm);
            1
        }
    })
});

export!(lsm_view_get_source_count(view: *const View) -> Result<c_uint> {
    Ok((*view).get_source_count() as c_uint)
});

export!(lsm_view_has_source_contents(view: *const View, src_id: c_uint) -> Result<c_int> {
    Ok(if (*view).get_source_contents(src_id as u32).is_some() { 1 } else { 0 })
});

export!(lsm_view_get_source_contents(
    view: *const View, src_id: c_uint, len_out: *mut c_uint,
    must_free: *mut c_int) -> Result<*mut u8>
{
    *must_free = 0;
    Ok(match (*view).get_source_contents(src_id as u32) {
        None => ptr::null_mut(),
        Some(contents) => {
            *len_out = contents.len() as c_uint;
            match contents {
                Cow::Borrowed(s) => s.as_ptr() as *mut u8,
                Cow::Owned(val) => {
                    *must_free = 1;
                    Box::into_raw(val.into_boxed_str()) as *mut u8
                }
            }
        }
    })
});

export!(lsm_view_get_source_name(
    view: *const View, src_id: c_uint, len_out: *mut c_uint) -> Result<*const u8>
{
    Ok(match (*view).get_source(src_id as u32) {
        None => ptr::null(),
        Some(name) => {
            *len_out = name.len() as c_uint;
            name.as_ptr()
        }
    })
});

export!(lsm_view_dump_memdb(
    view: *mut View, len_out: *mut c_uint, with_source_contents: c_int,
    with_names: c_int) -> Result<*mut u8>
{
    let memdb = (*view).dump_memdb(DumpOptions {
        with_source_contents: with_source_contents != 0,
        with_names: with_names != 0,
    })?;
    *len_out = memdb.len() as c_uint;
    Ok(Box::into_raw(memdb.into_boxed_slice()) as *mut u8)
});

export!(lsm_buffer_free(buf: *mut u8) {
    if !buf.is_null() {
        Box::from_raw(buf);
    }
});

export!(lsm_index_from_json(bytes: *const u8, len: c_uint) -> Result<*mut Index> {
    resultbox(Index::json_from_slice(slice::from_raw_parts(
        bytes,
        len as usize
    ))?)
});

export!(lsm_index_free(idx: *mut Index) {
    if !idx.is_null() {
        Box::from_raw(idx);
    }
});

export!(lsm_index_can_flatten(idx: *const Index) -> Result<c_int> {
    Ok(if (*idx).can_flatten() { 1 } else { 0 })
});

export!(lsm_index_into_view(idx: *mut Index) -> Result<*mut View> {
    resultbox(Box::from_raw(idx).into_view()?)
});

export!(lsm_view_or_index_from_json(
    bytes: *const u8, len: c_uint, view_out: *mut *mut View,
    idx_out: *mut *mut Index) -> Result<c_int> {
    match ViewOrIndex::from_slice(slice::from_raw_parts(
        bytes,
        len as usize
    ))? {
        ViewOrIndex::View(view) => {
            *view_out = Box::into_raw(Box::new(view));
            *idx_out = ptr::null_mut();
            Ok(1)
        }
        ViewOrIndex::Index(idx) => {
            *view_out = ptr::null_mut();
            *idx_out = Box::into_raw(Box::new(idx));
            Ok(2)
        }
    }
});
