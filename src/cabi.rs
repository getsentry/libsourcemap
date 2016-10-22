use std::ptr;
use std::slice;
use std::panic;
use std::ffi::CStr;
use std::os::raw::{c_int, c_uint, c_char};

use sourcemap::Error as SourceMapError;
use errors::{Error, ErrorKind, Result};
use unified::{View, TokenMatch, Index, ViewOrIndex};
use memdb::DumpOptions;


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
    pub code: c_int,
}

fn get_error_code_from_kind(kind: &ErrorKind) -> c_int {
    match *kind {
        ErrorKind::SourceMapError(SourceMapError::IndexedSourcemap) => 2,
        ErrorKind::SourceMapError(SourceMapError::BadJson(_, _, _)) => 3,
        ErrorKind::SourceMapError(SourceMapError::CannotFlatten(_)) => 4,
        ErrorKind::UnsupportedMemDbVersion => 5,
        ErrorKind::IoError(_) => 6,
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
    (*out).src = tm.src.as_ptr();
    (*out).src_len = tm.src.as_bytes().len() as c_uint;
    (*out).src_id = tm.src_id;
}


unsafe fn notify_err(err: Error, err_out: *mut CError) {
    if !err_out.is_null() {
        let s = format!("{}\x00", err);
        (*err_out).message = Box::into_raw(s.into_boxed_str()) as *mut u8;
        (*err_out).code = get_error_code_from_kind(err.kind());
    }
}

unsafe fn landingpad<F: FnOnce() -> Result<T> + panic::UnwindSafe, T>(
    f: F, err_out: *mut CError, err_rv: T) -> T
{
    match panic::catch_unwind(f) {
        Ok(rv) => {
            rv.map_err(|err| notify_err(err, err_out)).unwrap_or(err_rv)
        }
        Err(_) => {
            notify_err(ErrorKind::InternalError.into(), err_out);
            err_rv
        }
    }
}

unsafe fn boxed_landingpad<F: FnOnce() -> Result<T>, T>(
    f: F, err_out: *mut CError) -> *mut T
    where F: panic::UnwindSafe
{
    landingpad(|| match f() {
        Ok(v) => Ok(Box::into_raw(Box::new(v))),
        Err(err) => Err(err),
    }, err_out, ptr::null_mut())
}

fn silent_panic_handler(_pi: &panic::PanicInfo) {
    // don't do anything here.  This disables the default printing of
    // panics to stderr which we really don't care about here.
}


#[no_mangle]
pub unsafe extern "C" fn lsm_init() {
    panic::set_hook(Box::new(silent_panic_handler));
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_from_json(bytes: *const u8, len: c_uint, err_out: *mut CError) -> *mut View {
    boxed_landingpad(|| {
        View::json_from_slice(slice::from_raw_parts(
            bytes,
            len as usize
        ))
    }, err_out)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_from_memdb(bytes: *const u8, len: c_uint, err_out: *mut CError) -> *mut View {
    // XXX: this currently copies because that's safer.  Consider improving this?
    boxed_landingpad(|| {
        View::memdb_from_vec(slice::from_raw_parts(
            bytes,
            len as usize
        ).to_vec())
    }, err_out)
}

unsafe fn load_memdb_from_path(path: &CStr) -> Result<View> {
    View::memdb_from_path(try!(path.to_str()))
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_from_memdb_file(path: *const c_char, err_out: *mut CError) -> *mut View {
    boxed_landingpad(|| {
        load_memdb_from_path(CStr::from_ptr(path))
    }, err_out)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_free(view: *mut View) {
    if !view.is_null() {
        Box::from_raw(view);
    }
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_get_token_count(view: *const View) -> c_uint {
    // XXX: this silences panics
    panic::catch_unwind(|| (*view).get_token_count() as c_uint).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_get_token(view: *const View, idx: c_uint, out: *mut Token) -> c_int {
    // XXX: this silences panics
    panic::catch_unwind(|| {
        match (*view).get_token(idx as u32) {
            None => 0,
            Some(tm) => {
                set_token(out, &tm);
                1
            }
        }
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_lookup_token(view: *const View, line: c_uint, col: c_uint,
                                    out: *mut Token) -> c_int {
    // XXX: this silences panics
    panic::catch_unwind(|| {
        match (*view).lookup_token(line, col) {
            None => 0,
            Some(tm) => {
                set_token(out, &tm);
                1
            }
        }
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_get_source_count(view: *const View) -> c_uint {
    (*view).get_source_count() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_has_source_contents(view: *const View, src_id: u32) -> c_int {
    // XXX: this silences panics
    panic::catch_unwind(|| {
        if (*view).get_source_contents(src_id).is_some() { 1 } else { 0 }
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_get_source_contents(
    view: *const View, src_id: u32, len_out: *mut u32)
-> *const u8
{
    // XXX: this silences panics
    panic::catch_unwind(|| {
        match (*view).get_source_contents(src_id) {
            None => ptr::null_mut(),
            Some(contents) => {
                *len_out = contents.len() as u32;
                Box::into_raw(contents.into_boxed_str()) as *mut u8
            }
        }
    }).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_get_source_name(view: *const View, src_id: u32, len_out: *mut u32) -> *const u8 {
    // XXX: this silences panics
    panic::catch_unwind(|| {
        match (*view).get_source(src_id) {
            None => ptr::null(),
            Some(name) => {
                *len_out = name.len() as u32;
                name.as_ptr()
            }
        }
    }).unwrap_or(ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_dump_memdb(view: *mut View, len_out: *mut c_uint,
                                             with_source_contents: c_int,
                                             with_names: c_int, err_out: *mut CError)
    -> *mut u8
{
    landingpad(|| {
        let memdb = try!((*view).dump_memdb(DumpOptions {
            with_source_contents: with_source_contents != 0,
            with_names: with_names != 0,
        }));
        *len_out = memdb.len() as c_uint;
        Ok(Box::into_raw(memdb.into_boxed_slice()) as *mut u8)
    }, err_out, ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn lsm_buffer_free(buf: *mut u8) {
    if !buf.is_null() {
        Box::from_raw(buf);
    }
}

#[no_mangle]
pub unsafe extern "C" fn lsm_index_from_json(bytes: *const u8, len: c_uint, err_out: *mut CError) -> *mut Index {
    boxed_landingpad(|| {
        Index::json_from_slice(slice::from_raw_parts(
            bytes,
            len as usize
        ))
    }, err_out)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_index_free(idx: *mut Index) {
    if !idx.is_null() {
        Box::from_raw(idx);
    }
}

#[no_mangle]
pub unsafe extern "C" fn lsm_index_can_flatten(idx: *const Index) -> c_int {
    panic::catch_unwind(|| {
        if (*idx).can_flatten() { 1 } else { 0 }
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_index_into_view(idx: *mut Index, err_out: *mut CError) -> *mut View {
    boxed_landingpad(|| {
        Box::from_raw(idx).into_view()
    }, err_out)
}

#[no_mangle]
pub unsafe extern "C" fn lsm_view_or_index_from_json(
    bytes: *const u8, len: c_uint, err_out: *mut CError, view_out: *mut *mut View,
    idx_out: *mut *mut Index) -> c_int
{
    landingpad(|| {
        match try!(ViewOrIndex::from_slice(slice::from_raw_parts(
            bytes,
            len as usize
        ))) {
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
    }, err_out, 0)
}
