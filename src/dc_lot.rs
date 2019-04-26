use c2rust_bitfields::BitfieldStruct;
use libc;

use crate::dc_chat::*;
use crate::dc_contact::*;
use crate::dc_context::dc_context_t;
use crate::dc_imap::dc_imap_t;
use crate::dc_jobthread::dc_jobthread_t;
use crate::dc_msg::*;
use crate::dc_smtp::dc_smtp_t;
use crate::dc_sqlite3::*;
use crate::dc_stock::*;
use crate::dc_tools::*;
use crate::types::*;
use crate::x::*;

/* * Structure behind dc_lot_t */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dc_lot_t {
    pub magic: uint32_t,
    pub text1_meaning: libc::c_int,
    pub text1: *mut libc::c_char,
    pub text2: *mut libc::c_char,
    pub timestamp: time_t,
    pub state: libc::c_int,
    pub id: uint32_t,
    pub fingerprint: *mut libc::c_char,
    pub invitenumber: *mut libc::c_char,
    pub auth: *mut libc::c_char,
}

/* *
 * @class dc_lot_t
 *
 * An object containing a set of values.
 * The meaning of the values is defined by the function returning the object.
 * Lot objects are created
 * eg. by dc_chatlist_get_summary() or dc_msg_get_summary().
 *
 * NB: _Lot_ is used in the meaning _heap_ here.
 */
#[no_mangle]
pub unsafe extern "C" fn dc_lot_new() -> *mut dc_lot_t {
    let mut lot: *mut dc_lot_t = 0 as *mut dc_lot_t;
    lot = calloc(
        1i32 as libc::c_ulong,
        ::std::mem::size_of::<dc_lot_t>() as libc::c_ulong,
    ) as *mut dc_lot_t;
    if lot.is_null() {
        exit(27i32);
    }
    (*lot).magic = 0x107107i32 as uint32_t;
    (*lot).text1_meaning = 0i32;
    return lot;
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_empty(mut lot: *mut dc_lot_t) {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return;
    }
    free((*lot).text1 as *mut libc::c_void);
    (*lot).text1 = 0 as *mut libc::c_char;
    (*lot).text1_meaning = 0i32;
    free((*lot).text2 as *mut libc::c_void);
    (*lot).text2 = 0 as *mut libc::c_char;
    free((*lot).fingerprint as *mut libc::c_void);
    (*lot).fingerprint = 0 as *mut libc::c_char;
    free((*lot).invitenumber as *mut libc::c_void);
    (*lot).invitenumber = 0 as *mut libc::c_char;
    free((*lot).auth as *mut libc::c_void);
    (*lot).auth = 0 as *mut libc::c_char;
    (*lot).timestamp = 0i32 as time_t;
    (*lot).state = 0i32;
    (*lot).id = 0i32 as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_unref(mut set: *mut dc_lot_t) {
    if set.is_null() || (*set).magic != 0x107107i32 as libc::c_uint {
        return;
    }
    dc_lot_empty(set);
    (*set).magic = 0i32 as uint32_t;
    free(set as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_get_text1(mut lot: *const dc_lot_t) -> *mut libc::c_char {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return 0 as *mut libc::c_char;
    }
    return dc_strdup_keep_null((*lot).text1);
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_get_text2(mut lot: *const dc_lot_t) -> *mut libc::c_char {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return 0 as *mut libc::c_char;
    }
    return dc_strdup_keep_null((*lot).text2);
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_get_text1_meaning(mut lot: *const dc_lot_t) -> libc::c_int {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return 0i32;
    }
    return (*lot).text1_meaning;
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_get_state(mut lot: *const dc_lot_t) -> libc::c_int {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return 0i32;
    }
    return (*lot).state;
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_get_id(mut lot: *const dc_lot_t) -> uint32_t {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return 0i32 as uint32_t;
    }
    return (*lot).id;
}
#[no_mangle]
pub unsafe extern "C" fn dc_lot_get_timestamp(mut lot: *const dc_lot_t) -> time_t {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint {
        return 0i32 as time_t;
    }
    return (*lot).timestamp;
}
/* library-internal */
/* in practice, the user additionally cuts the string himself pixel-accurate */
#[no_mangle]
pub unsafe extern "C" fn dc_lot_fill(
    mut lot: *mut dc_lot_t,
    mut msg: *const dc_msg_t,
    mut chat: *const dc_chat_t,
    mut contact: *const dc_contact_t,
    mut context: *mut dc_context_t,
) {
    if lot.is_null() || (*lot).magic != 0x107107i32 as libc::c_uint || msg.is_null() {
        return;
    }
    if (*msg).state == 19i32 {
        (*lot).text1 = dc_stock_str(context, 3i32);
        (*lot).text1_meaning = 1i32
    } else if (*msg).from_id == 1i32 as libc::c_uint {
        if 0 != dc_msg_is_info(msg) || 0 != dc_chat_is_self_talk(chat) {
            (*lot).text1 = 0 as *mut libc::c_char;
            (*lot).text1_meaning = 0i32
        } else {
            (*lot).text1 = dc_stock_str(context, 2i32);
            (*lot).text1_meaning = 3i32
        }
    } else if chat.is_null() {
        (*lot).text1 = 0 as *mut libc::c_char;
        (*lot).text1_meaning = 0i32
    } else if (*chat).type_0 == 120i32 || (*chat).type_0 == 130i32 {
        if 0 != dc_msg_is_info(msg) || contact.is_null() {
            (*lot).text1 = 0 as *mut libc::c_char;
            (*lot).text1_meaning = 0i32
        } else {
            if !chat.is_null() && (*chat).id == 1i32 as libc::c_uint {
                (*lot).text1 = dc_contact_get_display_name(contact)
            } else {
                (*lot).text1 = dc_contact_get_first_name(contact)
            }
            (*lot).text1_meaning = 2i32
        }
    }
    (*lot).text2 =
        dc_msg_get_summarytext_by_raw((*msg).type_0, (*msg).text, (*msg).param, 160i32, context);
    (*lot).timestamp = dc_msg_get_timestamp(msg);
    (*lot).state = (*msg).state;
}