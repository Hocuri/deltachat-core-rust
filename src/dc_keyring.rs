use c2rust_bitfields::BitfieldStruct;
use libc;

use crate::dc_context::dc_context_t;
use crate::dc_key::*;
use crate::dc_lot::dc_lot_t;
use crate::dc_sqlite3::*;
use crate::types::*;
use crate::x::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dc_keyring_t {
    pub keys: *mut *mut dc_key_t,
    pub count: libc::c_int,
    pub allocated: libc::c_int,
}

#[no_mangle]
pub unsafe extern "C" fn dc_keyring_new() -> *mut dc_keyring_t {
    let mut keyring: *mut dc_keyring_t = 0 as *mut dc_keyring_t;
    keyring = calloc(
        1i32 as libc::c_ulong,
        ::std::mem::size_of::<dc_keyring_t>() as libc::c_ulong,
    ) as *mut dc_keyring_t;
    if keyring.is_null() {
        exit(42i32);
    }
    return keyring;
}
#[no_mangle]
pub unsafe extern "C" fn dc_keyring_unref(mut keyring: *mut dc_keyring_t) {
    if keyring.is_null() {
        return;
    }
    let mut i: libc::c_int = 0i32;
    while i < (*keyring).count {
        dc_key_unref(*(*keyring).keys.offset(i as isize));
        i += 1
    }
    free((*keyring).keys as *mut libc::c_void);
    free(keyring as *mut libc::c_void);
}
/* the reference counter of the key is increased by one */
#[no_mangle]
pub unsafe extern "C" fn dc_keyring_add(mut keyring: *mut dc_keyring_t, mut to_add: *mut dc_key_t) {
    if keyring.is_null() || to_add.is_null() {
        return;
    }
    if (*keyring).count == (*keyring).allocated {
        let mut newsize: libc::c_int = (*keyring).allocated * 2i32 + 10i32;
        (*keyring).keys = realloc(
            (*keyring).keys as *mut libc::c_void,
            (newsize as libc::c_ulong)
                .wrapping_mul(::std::mem::size_of::<*mut dc_key_t>() as libc::c_ulong),
        ) as *mut *mut dc_key_t;
        if (*keyring).keys.is_null() {
            exit(41i32);
        }
        (*keyring).allocated = newsize
    }
    let ref mut fresh0 = *(*keyring).keys.offset((*keyring).count as isize);
    *fresh0 = dc_key_ref(to_add);
    (*keyring).count += 1;
}
#[no_mangle]
pub unsafe extern "C" fn dc_keyring_load_self_private_for_decrypting(
    mut keyring: *mut dc_keyring_t,
    mut self_addr: *const libc::c_char,
    mut sql: *mut dc_sqlite3_t,
) -> libc::c_int {
    if keyring.is_null() || self_addr.is_null() || sql.is_null() {
        return 0i32;
    }
    let mut stmt: *mut sqlite3_stmt = dc_sqlite3_prepare(
        sql,
        b"SELECT private_key FROM keypairs ORDER BY addr=? DESC, is_default DESC;\x00" as *const u8
            as *const libc::c_char,
    );
    sqlite3_bind_text(stmt, 1i32, self_addr, -1i32, None);
    while sqlite3_step(stmt) == 100i32 {
        let mut key: *mut dc_key_t = dc_key_new();
        if 0 != dc_key_set_from_stmt(key, stmt, 0i32, 1i32) {
            dc_keyring_add(keyring, key);
        }
        dc_key_unref(key);
    }
    sqlite3_finalize(stmt);
    return 1i32;
}