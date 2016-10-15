use std::ptr;
use std::mem;
use std::slice;
use std::os::raw::{c_int, c_uint};

use sourcemap::Error as SourceMapError;
use errors::{Error, ErrorKind};
use unified::{View, TokenMatch};


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
        ErrorKind::UnsupportedMemDbVersion => 4,
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


unsafe fn notify_err<T>(err: Error, err_out: *mut CError) -> *mut T {
    if !err_out.is_null() {
        let s = format!("{}\x00", err);
        (*err_out).message = Box::into_raw(s.into_boxed_str()) as *mut u8;
        (*err_out).code = get_error_code_from_kind(err.kind());
    }
    0 as *mut T
}

#[no_mangle]
pub unsafe fn lsm_view_from_json(bytes: *const u8, len: c_uint, err_out: *mut CError) -> *mut View {
    match View::json_from_slice(slice::from_raw_parts(
        mem::transmute(bytes),
        len as usize
    )) {
        Ok(v) => Box::into_raw(Box::new(v)),
        Err(err) => notify_err(err, err_out)
    }
}

#[no_mangle]
pub unsafe fn lsm_view_from_memdb(bytes: *const u8, len: c_uint, err_out: *mut CError) -> *mut View {
    // XXX: this currently copies because that's safer.  Consider improving this?
    match View::memdb_from_vec(slice::from_raw_parts(
        mem::transmute(bytes),
        len as usize
    ).to_vec()) {
        Ok(v) => Box::into_raw(Box::new(v)),
        Err(err) => notify_err(err, err_out)
    }
}

#[no_mangle]
pub unsafe fn lsm_view_free(view: *mut View) {
    if !view.is_null() {
        Box::from_raw(view);
    }
}

#[no_mangle]
pub unsafe fn lsm_view_get_token_count(view: *const View) -> c_uint {
    (*view).get_token_count() as c_uint
}

#[no_mangle]
pub unsafe fn lsm_view_get_token(view: *const View, idx: c_uint, out: *mut Token) -> c_int {
    match (*view).get_token(idx as u32) {
        None => 0,
        Some(tm) => {
            set_token(out, &tm);
            1
        }
    }
}

#[no_mangle]
pub unsafe fn lsm_view_lookup_token(view: *const View, line: c_uint, col: c_uint,
                                    out: *mut Token) -> c_int {
    match (*view).lookup_token(line, col) {
        None => 0,
        Some(tm) => {
            set_token(out, &tm);
            1
        }
    }
}

#[no_mangle]
pub unsafe fn lsm_view_get_source_contents(view: *const View, src_id: u32, len_out: *mut u32) -> *const u8 {
    match (*view).get_source_contents(src_id) {
        None => ptr::null(),
        Some(contents) => {
            *len_out = contents.len() as u32;
            contents.as_ptr()
        }
    }
}

#[no_mangle]
pub unsafe fn lsm_view_dump_memdb(view: *mut View, len_out: *mut c_uint) -> *mut u8 {
    let memdb = (*view).dump_memdb();
    *len_out = memdb.len() as c_uint;
    Box::into_raw(memdb.into_boxed_slice()) as *mut u8
}

#[no_mangle]
pub unsafe fn lsm_buffer_free(buf: *mut u8) {
    if !buf.is_null() {
        Box::from_raw(buf);
    }
}
