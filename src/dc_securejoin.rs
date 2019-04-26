use c2rust_bitfields::BitfieldStruct;
use libc;

use crate::dc_apeerstate::*;
use crate::dc_array::*;
use crate::dc_chat::*;
use crate::dc_configure::*;
use crate::dc_contact::*;
use crate::dc_context::dc_context_t;
use crate::dc_e2ee::*;
use crate::dc_hash::*;
use crate::dc_imap::dc_imap_t;
use crate::dc_key::*;
use crate::dc_log::*;
use crate::dc_lot::*;
use crate::dc_mimeparser::*;
use crate::dc_msg::*;
use crate::dc_param::*;
use crate::dc_qr::*;
use crate::dc_sqlite3::*;
use crate::dc_stock::*;
use crate::dc_stock::*;
use crate::dc_strencode::*;
use crate::dc_token::*;
use crate::dc_tools::*;
use crate::types::*;
use crate::x::*;

#[no_mangle]
pub unsafe extern "C" fn dc_get_securejoin_qr(
    mut context: *mut dc_context_t,
    mut group_chat_id: uint32_t,
) -> *mut libc::c_char {
    let mut current_block: u64;
    /* =========================================================
    ====             Alice - the inviter side            ====
    ====   Step 1 in "Setup verified contact" protocol   ====
    ========================================================= */
    let mut qr: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut self_addr: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut self_addr_urlencoded: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut self_name: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut self_name_urlencoded: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut fingerprint: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut invitenumber: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut auth: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut chat: *mut dc_chat_t = 0 as *mut dc_chat_t;
    let mut group_name: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut group_name_urlencoded: *mut libc::c_char = 0 as *mut libc::c_char;
    if !(context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint) {
        dc_ensure_secret_key_exists(context);
        invitenumber = dc_token_lookup(context, DC_TOKEN_INVITENUMBER, group_chat_id);
        if invitenumber.is_null() {
            invitenumber = dc_create_id();
            dc_token_save(context, DC_TOKEN_INVITENUMBER, group_chat_id, invitenumber);
        }
        auth = dc_token_lookup(context, DC_TOKEN_AUTH, group_chat_id);
        if auth.is_null() {
            auth = dc_create_id();
            dc_token_save(context, DC_TOKEN_AUTH, group_chat_id, auth);
        }
        self_addr = dc_sqlite3_get_config(
            (*context).sql,
            b"configured_addr\x00" as *const u8 as *const libc::c_char,
            0 as *const libc::c_char,
        );
        if self_addr.is_null() {
            dc_log_error(
                context,
                0i32,
                b"Not configured, cannot generate QR code.\x00" as *const u8 as *const libc::c_char,
            );
        } else {
            self_name = dc_sqlite3_get_config(
                (*context).sql,
                b"displayname\x00" as *const u8 as *const libc::c_char,
                b"\x00" as *const u8 as *const libc::c_char,
            );
            fingerprint = get_self_fingerprint(context);
            if !fingerprint.is_null() {
                self_addr_urlencoded = dc_urlencode(self_addr);
                self_name_urlencoded = dc_urlencode(self_name);
                if 0 != group_chat_id {
                    chat = dc_get_chat(context, group_chat_id);
                    if chat.is_null() {
                        dc_log_error(
                            context,
                            0i32,
                            b"Cannot get QR-code for chat-id %i\x00" as *const u8
                                as *const libc::c_char,
                            group_chat_id,
                        );
                        current_block = 9531737720721467826;
                    } else {
                        group_name = dc_chat_get_name(chat);
                        group_name_urlencoded = dc_urlencode(group_name);
                        qr = dc_mprintf(
                            b"OPENPGP4FPR:%s#a=%s&g=%s&x=%s&i=%s&s=%s\x00" as *const u8
                                as *const libc::c_char,
                            fingerprint,
                            self_addr_urlencoded,
                            group_name_urlencoded,
                            (*chat).grpid,
                            invitenumber,
                            auth,
                        );
                        current_block = 1118134448028020070;
                    }
                } else {
                    qr = dc_mprintf(
                        b"OPENPGP4FPR:%s#a=%s&n=%s&i=%s&s=%s\x00" as *const u8
                            as *const libc::c_char,
                        fingerprint,
                        self_addr_urlencoded,
                        self_name_urlencoded,
                        invitenumber,
                        auth,
                    );
                    current_block = 1118134448028020070;
                }
                match current_block {
                    9531737720721467826 => {}
                    _ => {
                        dc_log_info(
                            context,
                            0i32,
                            b"Generated QR code: %s\x00" as *const u8 as *const libc::c_char,
                            qr,
                        );
                    }
                }
            }
        }
    }
    free(self_addr_urlencoded as *mut libc::c_void);
    free(self_addr as *mut libc::c_void);
    free(self_name as *mut libc::c_void);
    free(self_name_urlencoded as *mut libc::c_void);
    free(fingerprint as *mut libc::c_void);
    free(invitenumber as *mut libc::c_void);
    free(auth as *mut libc::c_void);
    dc_chat_unref(chat);
    free(group_name as *mut libc::c_void);
    free(group_name_urlencoded as *mut libc::c_void);
    return if !qr.is_null() {
        qr
    } else {
        dc_strdup(0 as *const libc::c_char)
    };
}
unsafe extern "C" fn get_self_fingerprint(mut context: *mut dc_context_t) -> *mut libc::c_char {
    let mut self_addr: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut self_key: *mut dc_key_t = dc_key_new();
    let mut fingerprint: *mut libc::c_char = 0 as *mut libc::c_char;
    self_addr = dc_sqlite3_get_config(
        (*context).sql,
        b"configured_addr\x00" as *const u8 as *const libc::c_char,
        0 as *const libc::c_char,
    );
    if !(self_addr.is_null() || 0 == dc_key_load_self_public(self_key, self_addr, (*context).sql)) {
        fingerprint = dc_key_get_fingerprint(self_key);
        fingerprint.is_null();
    }
    free(self_addr as *mut libc::c_void);
    dc_key_unref(self_key);
    return fingerprint;
}
#[no_mangle]
pub unsafe extern "C" fn dc_join_securejoin(
    mut context: *mut dc_context_t,
    mut qr: *const libc::c_char,
) -> uint32_t {
    /* ==========================================================
    ====             Bob - the joiner's side             =====
    ====   Step 2 in "Setup verified contact" protocol   =====
    ========================================================== */
    let mut ret_chat_id: libc::c_int = 0i32;
    let mut ongoing_allocated: libc::c_int = 0i32;
    let mut contact_chat_id: uint32_t = 0i32 as uint32_t;
    let mut join_vg: libc::c_int = 0i32;
    let mut qr_scan: *mut dc_lot_t = 0 as *mut dc_lot_t;
    let mut qr_locked: libc::c_int = 0i32;
    dc_log_info(
        context,
        0i32,
        b"Requesting secure-join ...\x00" as *const u8 as *const libc::c_char,
    );
    dc_ensure_secret_key_exists(context);
    ongoing_allocated = dc_alloc_ongoing(context);
    if !(ongoing_allocated == 0i32) {
        qr_scan = dc_check_qr(context, qr);
        if qr_scan.is_null() || (*qr_scan).state != 200i32 && (*qr_scan).state != 202i32 {
            dc_log_error(
                context,
                0i32,
                b"Unknown QR code.\x00" as *const u8 as *const libc::c_char,
            );
        } else {
            contact_chat_id = dc_create_chat_by_contact_id(context, (*qr_scan).id);
            if contact_chat_id == 0i32 as libc::c_uint {
                dc_log_error(
                    context,
                    0i32,
                    b"Unknown contact.\x00" as *const u8 as *const libc::c_char,
                );
            } else if !(0 != (*context).shall_stop_ongoing) {
                join_vg = ((*qr_scan).state == 202i32) as libc::c_int;
                (*context).bobs_status = 0i32;
                pthread_mutex_lock(&mut (*context).bobs_qr_critical);
                qr_locked = 1i32;
                (*context).bobs_qr_scan = qr_scan;
                if 0 != qr_locked {
                    pthread_mutex_unlock(&mut (*context).bobs_qr_critical);
                    qr_locked = 0i32
                }
                if 0 != fingerprint_equals_sender(context, (*qr_scan).fingerprint, contact_chat_id)
                {
                    dc_log_info(
                        context,
                        0i32,
                        b"Taking protocol shortcut.\x00" as *const u8 as *const libc::c_char,
                    );
                    (*context).bob_expects = 6i32;
                    (*context).cb.expect("non-null function pointer")(
                        context,
                        2061i32,
                        chat_id_2_contact_id(context, contact_chat_id) as uintptr_t,
                        400i32 as uintptr_t,
                    );
                    let mut own_fingerprint: *mut libc::c_char = get_self_fingerprint(context);
                    send_handshake_msg(
                        context,
                        contact_chat_id,
                        if 0 != join_vg {
                            b"vg-request-with-auth\x00" as *const u8 as *const libc::c_char
                        } else {
                            b"vc-request-with-auth\x00" as *const u8 as *const libc::c_char
                        },
                        (*qr_scan).auth,
                        own_fingerprint,
                        if 0 != join_vg {
                            (*qr_scan).text2
                        } else {
                            0 as *mut libc::c_char
                        },
                    );
                    free(own_fingerprint as *mut libc::c_void);
                } else {
                    (*context).bob_expects = 2i32;
                    send_handshake_msg(
                        context,
                        contact_chat_id,
                        if 0 != join_vg {
                            b"vg-request\x00" as *const u8 as *const libc::c_char
                        } else {
                            b"vc-request\x00" as *const u8 as *const libc::c_char
                        },
                        (*qr_scan).invitenumber,
                        0 as *const libc::c_char,
                        0 as *const libc::c_char,
                    );
                }
                // Bob -> Alice
                while !(0 != (*context).shall_stop_ongoing) {
                    usleep((300i32 * 1000i32) as useconds_t);
                }
            }
        }
    }
    (*context).bob_expects = 0i32;
    if (*context).bobs_status == 1i32 {
        if 0 != join_vg {
            ret_chat_id = dc_get_chat_id_by_grpid(
                context,
                (*qr_scan).text2,
                0 as *mut libc::c_int,
                0 as *mut libc::c_int,
            ) as libc::c_int
        } else {
            ret_chat_id = contact_chat_id as libc::c_int
        }
    }
    pthread_mutex_lock(&mut (*context).bobs_qr_critical);
    qr_locked = 1i32;
    (*context).bobs_qr_scan = 0 as *mut dc_lot_t;
    if 0 != qr_locked {
        pthread_mutex_unlock(&mut (*context).bobs_qr_critical);
        qr_locked = 0i32
    }
    dc_lot_unref(qr_scan);
    if 0 != ongoing_allocated {
        dc_free_ongoing(context);
    }
    return ret_chat_id as uint32_t;
}
unsafe extern "C" fn send_handshake_msg(
    mut context: *mut dc_context_t,
    mut contact_chat_id: uint32_t,
    mut step: *const libc::c_char,
    mut param2: *const libc::c_char,
    mut fingerprint: *const libc::c_char,
    mut grpid: *const libc::c_char,
) {
    let mut msg: *mut dc_msg_t = dc_msg_new_untyped(context);
    (*msg).type_0 = 10i32;
    (*msg).text = dc_mprintf(
        b"Secure-Join: %s\x00" as *const u8 as *const libc::c_char,
        step,
    );
    (*msg).hidden = 1i32;
    dc_param_set_int((*msg).param, 'S' as i32, 7i32);
    dc_param_set((*msg).param, 'E' as i32, step);
    if !param2.is_null() {
        dc_param_set((*msg).param, 'F' as i32, param2);
    }
    if !fingerprint.is_null() {
        dc_param_set((*msg).param, 'G' as i32, fingerprint);
    }
    if !grpid.is_null() {
        dc_param_set((*msg).param, 'H' as i32, grpid);
    }
    if strcmp(step, b"vg-request\x00" as *const u8 as *const libc::c_char) == 0i32
        || strcmp(step, b"vc-request\x00" as *const u8 as *const libc::c_char) == 0i32
    {
        dc_param_set_int((*msg).param, 'u' as i32, 1i32);
    } else {
        dc_param_set_int((*msg).param, 'c' as i32, 1i32);
    }
    dc_send_msg(context, contact_chat_id, msg);
    dc_msg_unref(msg);
}
unsafe extern "C" fn chat_id_2_contact_id(
    mut context: *mut dc_context_t,
    mut contact_chat_id: uint32_t,
) -> uint32_t {
    let mut contact_id: uint32_t = 0i32 as uint32_t;
    let mut contacts: *mut dc_array_t = dc_get_chat_contacts(context, contact_chat_id);
    if !(dc_array_get_cnt(contacts) != 1i32 as libc::c_ulong) {
        contact_id = dc_array_get_id(contacts, 0i32 as size_t)
    }
    dc_array_unref(contacts);
    return contact_id;
}
unsafe extern "C" fn fingerprint_equals_sender(
    mut context: *mut dc_context_t,
    mut fingerprint: *const libc::c_char,
    mut contact_chat_id: uint32_t,
) -> libc::c_int {
    let mut fingerprint_equal: libc::c_int = 0i32;
    let mut contacts: *mut dc_array_t = dc_get_chat_contacts(context, contact_chat_id);
    let mut contact: *mut dc_contact_t = dc_contact_new(context);
    let mut peerstate: *mut dc_apeerstate_t = dc_apeerstate_new(context);
    let mut fingerprint_normalized: *mut libc::c_char = 0 as *mut libc::c_char;
    if !(dc_array_get_cnt(contacts) != 1i32 as libc::c_ulong) {
        if !(0
            == dc_contact_load_from_db(
                contact,
                (*context).sql,
                dc_array_get_id(contacts, 0i32 as size_t),
            )
            || 0 == dc_apeerstate_load_by_addr(peerstate, (*context).sql, (*contact).addr))
        {
            fingerprint_normalized = dc_normalize_fingerprint(fingerprint);
            if strcasecmp(fingerprint_normalized, (*peerstate).public_key_fingerprint) == 0i32 {
                fingerprint_equal = 1i32
            }
        }
    }
    free(fingerprint_normalized as *mut libc::c_void);
    dc_contact_unref(contact);
    dc_array_unref(contacts);
    return fingerprint_equal;
}
/* library private: secure-join */
#[no_mangle]
pub unsafe extern "C" fn dc_handle_securejoin_handshake(
    mut context: *mut dc_context_t,
    mut mimeparser: *mut dc_mimeparser_t,
    mut contact_id: uint32_t,
) -> libc::c_int {
    let mut current_block: u64;
    let mut qr_locked: libc::c_int = 0i32;
    let mut step: *const libc::c_char = 0 as *const libc::c_char;
    let mut join_vg: libc::c_int = 0i32;
    let mut scanned_fingerprint_of_alice: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut auth: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut own_fingerprint: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut contact_chat_id: uint32_t = 0i32 as uint32_t;
    let mut contact_chat_id_blocked: libc::c_int = 0i32;
    let mut grpid: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut ret: libc::c_int = 0i32;
    let mut contact: *mut dc_contact_t = 0 as *mut dc_contact_t;
    if !(context.is_null() || mimeparser.is_null() || contact_id <= 9i32 as libc::c_uint) {
        step = lookup_field(
            mimeparser,
            b"Secure-Join\x00" as *const u8 as *const libc::c_char,
        );
        if !step.is_null() {
            dc_log_info(
                context,
                0i32,
                b">>>>>>>>>>>>>>>>>>>>>>>>> secure-join message \'%s\' received\x00" as *const u8
                    as *const libc::c_char,
                step,
            );
            join_vg = (strncmp(
                step,
                b"vg-\x00" as *const u8 as *const libc::c_char,
                3i32 as libc::c_ulong,
            ) == 0i32) as libc::c_int;
            dc_create_or_lookup_nchat_by_contact_id(
                context,
                contact_id,
                0i32,
                &mut contact_chat_id,
                &mut contact_chat_id_blocked,
            );
            if 0 != contact_chat_id_blocked {
                dc_unblock_chat(context, contact_chat_id);
            }
            ret = 0x2i32;
            if strcmp(step, b"vg-request\x00" as *const u8 as *const libc::c_char) == 0i32
                || strcmp(step, b"vc-request\x00" as *const u8 as *const libc::c_char) == 0i32
            {
                /* =========================================================
                ====             Alice - the inviter side            ====
                ====   Step 3 in "Setup verified contact" protocol   ====
                ========================================================= */
                // this message may be unencrypted (Bob, the joinder and the sender, might not have Alice's key yet)
                // it just ensures, we have Bobs key now. If we do _not_ have the key because eg. MitM has removed it,
                // send_message() will fail with the error "End-to-end-encryption unavailable unexpectedly.", so, there is no additional check needed here.
                // verify that the `Secure-Join-Invitenumber:`-header matches invitenumber written to the QR code
                let mut invitenumber: *const libc::c_char = 0 as *const libc::c_char;
                invitenumber = lookup_field(
                    mimeparser,
                    b"Secure-Join-Invitenumber\x00" as *const u8 as *const libc::c_char,
                );
                if invitenumber.is_null() {
                    dc_log_warning(
                        context,
                        0i32,
                        b"Secure-join denied (invitenumber missing).\x00" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else if dc_token_exists(context, DC_TOKEN_INVITENUMBER, invitenumber) == 0i32 {
                    dc_log_warning(
                        context,
                        0i32,
                        b"Secure-join denied (bad invitenumber).\x00" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else {
                    dc_log_info(
                        context,
                        0i32,
                        b"Secure-join requested.\x00" as *const u8 as *const libc::c_char,
                    );
                    (*context).cb.expect("non-null function pointer")(
                        context,
                        2060i32,
                        contact_id as uintptr_t,
                        300i32 as uintptr_t,
                    );
                    send_handshake_msg(
                        context,
                        contact_chat_id,
                        if 0 != join_vg {
                            b"vg-auth-required\x00" as *const u8 as *const libc::c_char
                        } else {
                            b"vc-auth-required\x00" as *const u8 as *const libc::c_char
                        },
                        0 as *const libc::c_char,
                        0 as *const libc::c_char,
                        0 as *const libc::c_char,
                    );
                    current_block = 10256747982273457880;
                }
            } else if strcmp(
                step,
                b"vg-auth-required\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
                || strcmp(
                    step,
                    b"vc-auth-required\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
            {
                pthread_mutex_lock(&mut (*context).bobs_qr_critical);
                qr_locked = 1i32;
                if (*context).bobs_qr_scan.is_null()
                    || (*context).bob_expects != 2i32
                    || 0 != join_vg && (*(*context).bobs_qr_scan).state != 202i32
                {
                    dc_log_warning(
                        context,
                        0i32,
                        b"auth-required message out of sync.\x00" as *const u8
                            as *const libc::c_char,
                    );
                    // no error, just aborted somehow or a mail from another handshake
                    current_block = 4378276786830486580;
                } else {
                    scanned_fingerprint_of_alice =
                        dc_strdup((*(*context).bobs_qr_scan).fingerprint);
                    auth = dc_strdup((*(*context).bobs_qr_scan).auth);
                    if 0 != join_vg {
                        grpid = dc_strdup((*(*context).bobs_qr_scan).text2)
                    }
                    if 0 != qr_locked {
                        pthread_mutex_unlock(&mut (*context).bobs_qr_critical);
                        qr_locked = 0i32
                    }
                    if 0 == encrypted_and_signed(mimeparser, scanned_fingerprint_of_alice) {
                        could_not_establish_secure_connection(
                            context,
                            contact_chat_id,
                            if 0 != (*(*mimeparser).e2ee_helper).encrypted {
                                b"No valid signature.\x00" as *const u8 as *const libc::c_char
                            } else {
                                b"Not encrypted.\x00" as *const u8 as *const libc::c_char
                            },
                        );
                        end_bobs_joining(context, 0i32);
                        current_block = 4378276786830486580;
                    } else if 0
                        == fingerprint_equals_sender(
                            context,
                            scanned_fingerprint_of_alice,
                            contact_chat_id,
                        )
                    {
                        could_not_establish_secure_connection(
                            context,
                            contact_chat_id,
                            b"Fingerprint mismatch on joiner-side.\x00" as *const u8
                                as *const libc::c_char,
                        );
                        end_bobs_joining(context, 0i32);
                        current_block = 4378276786830486580;
                    } else {
                        dc_log_info(
                            context,
                            0i32,
                            b"Fingerprint verified.\x00" as *const u8 as *const libc::c_char,
                        );
                        own_fingerprint = get_self_fingerprint(context);
                        (*context).cb.expect("non-null function pointer")(
                            context,
                            2061i32,
                            contact_id as uintptr_t,
                            400i32 as uintptr_t,
                        );
                        (*context).bob_expects = 6i32;
                        send_handshake_msg(
                            context,
                            contact_chat_id,
                            if 0 != join_vg {
                                b"vg-request-with-auth\x00" as *const u8 as *const libc::c_char
                            } else {
                                b"vc-request-with-auth\x00" as *const u8 as *const libc::c_char
                            },
                            auth,
                            own_fingerprint,
                            grpid,
                        );
                        current_block = 10256747982273457880;
                    }
                }
            } else if strcmp(
                step,
                b"vg-request-with-auth\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
                || strcmp(
                    step,
                    b"vc-request-with-auth\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
            {
                /* ============================================================
                ====              Alice - the inviter side              ====
                ====   Steps 5+6 in "Setup verified contact" protocol   ====
                ====  Step 6 in "Out-of-band verified groups" protocol  ====
                ============================================================ */
                // verify that Secure-Join-Fingerprint:-header matches the fingerprint of Bob
                let mut fingerprint: *const libc::c_char = 0 as *const libc::c_char;
                fingerprint = lookup_field(
                    mimeparser,
                    b"Secure-Join-Fingerprint\x00" as *const u8 as *const libc::c_char,
                );
                if fingerprint.is_null() {
                    could_not_establish_secure_connection(
                        context,
                        contact_chat_id,
                        b"Fingerprint not provided.\x00" as *const u8 as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else if 0 == encrypted_and_signed(mimeparser, fingerprint) {
                    could_not_establish_secure_connection(
                        context,
                        contact_chat_id,
                        b"Auth not encrypted.\x00" as *const u8 as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else if 0 == fingerprint_equals_sender(context, fingerprint, contact_chat_id) {
                    could_not_establish_secure_connection(
                        context,
                        contact_chat_id,
                        b"Fingerprint mismatch on inviter-side.\x00" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else {
                    dc_log_info(
                        context,
                        0i32,
                        b"Fingerprint verified.\x00" as *const u8 as *const libc::c_char,
                    );
                    // verify that the `Secure-Join-Auth:`-header matches the secret written to the QR code
                    let mut auth_0: *const libc::c_char = 0 as *const libc::c_char;
                    auth_0 = lookup_field(
                        mimeparser,
                        b"Secure-Join-Auth\x00" as *const u8 as *const libc::c_char,
                    );
                    if auth_0.is_null() {
                        could_not_establish_secure_connection(
                            context,
                            contact_chat_id,
                            b"Auth not provided.\x00" as *const u8 as *const libc::c_char,
                        );
                        current_block = 4378276786830486580;
                    } else if dc_token_exists(context, DC_TOKEN_AUTH, auth_0) == 0i32 {
                        could_not_establish_secure_connection(
                            context,
                            contact_chat_id,
                            b"Auth invalid.\x00" as *const u8 as *const libc::c_char,
                        );
                        current_block = 4378276786830486580;
                    } else if 0 == mark_peer_as_verified(context, fingerprint) {
                        could_not_establish_secure_connection(
                            context,
                            contact_chat_id,
                            b"Fingerprint mismatch on inviter-side.\x00" as *const u8
                                as *const libc::c_char,
                        );
                        current_block = 4378276786830486580;
                    } else {
                        dc_scaleup_contact_origin(context, contact_id, 0x1000000i32);
                        dc_log_info(
                            context,
                            0i32,
                            b"Auth verified.\x00" as *const u8 as *const libc::c_char,
                        );
                        secure_connection_established(context, contact_chat_id);
                        (*context).cb.expect("non-null function pointer")(
                            context,
                            2030i32,
                            contact_id as uintptr_t,
                            0i32 as uintptr_t,
                        );
                        (*context).cb.expect("non-null function pointer")(
                            context,
                            2060i32,
                            contact_id as uintptr_t,
                            600i32 as uintptr_t,
                        );
                        if 0 != join_vg {
                            grpid = dc_strdup(lookup_field(
                                mimeparser,
                                b"Secure-Join-Group\x00" as *const u8 as *const libc::c_char,
                            ));
                            let mut group_chat_id: uint32_t = dc_get_chat_id_by_grpid(
                                context,
                                grpid,
                                0 as *mut libc::c_int,
                                0 as *mut libc::c_int,
                            );
                            if group_chat_id == 0i32 as libc::c_uint {
                                dc_log_error(
                                    context,
                                    0i32,
                                    b"Chat %s not found.\x00" as *const u8 as *const libc::c_char,
                                    grpid,
                                );
                                current_block = 4378276786830486580;
                            } else {
                                dc_add_contact_to_chat_ex(
                                    context,
                                    group_chat_id,
                                    contact_id,
                                    0x1i32,
                                );
                                current_block = 10256747982273457880;
                            }
                        } else {
                            send_handshake_msg(
                                context,
                                contact_chat_id,
                                b"vc-contact-confirm\x00" as *const u8 as *const libc::c_char,
                                0 as *const libc::c_char,
                                0 as *const libc::c_char,
                                0 as *const libc::c_char,
                            );
                            (*context).cb.expect("non-null function pointer")(
                                context,
                                2060i32,
                                contact_id as uintptr_t,
                                1000i32 as uintptr_t,
                            );
                            current_block = 10256747982273457880;
                        }
                    }
                }
            } else if strcmp(
                step,
                b"vg-member-added\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
                || strcmp(
                    step,
                    b"vc-contact-confirm\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
            {
                if 0 != join_vg {
                    ret = 0x1i32
                }
                if (*context).bob_expects != 6i32 {
                    dc_log_info(
                        context,
                        0i32,
                        b"Message belongs to a different handshake.\x00" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else {
                    pthread_mutex_lock(&mut (*context).bobs_qr_critical);
                    qr_locked = 1i32;
                    if (*context).bobs_qr_scan.is_null()
                        || 0 != join_vg && (*(*context).bobs_qr_scan).state != 202i32
                    {
                        dc_log_warning(
                            context,
                            0i32,
                            b"Message out of sync or belongs to a different handshake.\x00"
                                as *const u8 as *const libc::c_char,
                        );
                        current_block = 4378276786830486580;
                    } else {
                        scanned_fingerprint_of_alice =
                            dc_strdup((*(*context).bobs_qr_scan).fingerprint);
                        if 0 != join_vg {
                            grpid = dc_strdup((*(*context).bobs_qr_scan).text2)
                        }
                        if 0 != qr_locked {
                            pthread_mutex_unlock(&mut (*context).bobs_qr_critical);
                            qr_locked = 0i32
                        }
                        let mut vg_expect_encrypted: libc::c_int = 1i32;
                        if 0 != join_vg {
                            let mut is_verified_group: libc::c_int = 0i32;
                            dc_get_chat_id_by_grpid(
                                context,
                                grpid,
                                0 as *mut libc::c_int,
                                &mut is_verified_group,
                            );
                            if 0 == is_verified_group {
                                vg_expect_encrypted = 0i32
                            }
                        }
                        if 0 != vg_expect_encrypted {
                            if 0 == encrypted_and_signed(mimeparser, scanned_fingerprint_of_alice) {
                                could_not_establish_secure_connection(
                                    context,
                                    contact_chat_id,
                                    b"Contact confirm message not encrypted.\x00" as *const u8
                                        as *const libc::c_char,
                                );
                                end_bobs_joining(context, 0i32);
                                current_block = 4378276786830486580;
                            } else {
                                current_block = 5195798230510548452;
                            }
                        } else {
                            current_block = 5195798230510548452;
                        }
                        match current_block {
                            4378276786830486580 => {}
                            _ => {
                                if 0 == mark_peer_as_verified(context, scanned_fingerprint_of_alice)
                                {
                                    could_not_establish_secure_connection(
                                        context,
                                        contact_chat_id,
                                        b"Fingerprint mismatch on joiner-side.\x00" as *const u8
                                            as *const libc::c_char,
                                    );
                                    current_block = 4378276786830486580;
                                } else {
                                    dc_scaleup_contact_origin(context, contact_id, 0x2000000i32);
                                    (*context).cb.expect("non-null function pointer")(
                                        context,
                                        2030i32,
                                        0i32 as uintptr_t,
                                        0i32 as uintptr_t,
                                    );
                                    if 0 != join_vg {
                                        if 0 == dc_addr_equals_self(
                                            context,
                                            lookup_field(
                                                mimeparser,
                                                b"Chat-Group-Member-Added\x00" as *const u8
                                                    as *const libc::c_char,
                                            ),
                                        ) {
                                            dc_log_info(context, 0i32,
                                                        b"Message belongs to a different handshake (scaled up contact anyway to allow creation of group).\x00"
                                                            as *const u8 as
                                                            *const libc::c_char);
                                            current_block = 4378276786830486580;
                                        } else {
                                            current_block = 9180031981464905198;
                                        }
                                    } else {
                                        current_block = 9180031981464905198;
                                    }
                                    match current_block {
                                        4378276786830486580 => {}
                                        _ => {
                                            secure_connection_established(context, contact_chat_id);
                                            (*context).bob_expects = 0i32;
                                            if 0 != join_vg {
                                                send_handshake_msg(
                                                    context,
                                                    contact_chat_id,
                                                    b"vg-member-added-received\x00" as *const u8
                                                        as *const libc::c_char,
                                                    0 as *const libc::c_char,
                                                    0 as *const libc::c_char,
                                                    0 as *const libc::c_char,
                                                );
                                            }
                                            end_bobs_joining(context, 1i32);
                                            current_block = 10256747982273457880;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if strcmp(
                step,
                b"vg-member-added-received\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
            {
                /* ============================================================
                ====              Alice - the inviter side              ====
                ====  Step 8 in "Out-of-band verified groups" protocol  ====
                ============================================================ */
                contact = dc_get_contact(context, contact_id);
                if contact.is_null() || 0 == dc_contact_is_verified(contact) {
                    dc_log_warning(
                        context,
                        0i32,
                        b"vg-member-added-received invalid.\x00" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 4378276786830486580;
                } else {
                    (*context).cb.expect("non-null function pointer")(
                        context,
                        2060i32,
                        contact_id as uintptr_t,
                        800i32 as uintptr_t,
                    );
                    (*context).cb.expect("non-null function pointer")(
                        context,
                        2060i32,
                        contact_id as uintptr_t,
                        1000i32 as uintptr_t,
                    );
                    current_block = 10256747982273457880;
                }
            } else {
                current_block = 10256747982273457880;
            }
            match current_block {
                4378276786830486580 => {}
                _ => {
                    if 0 != ret & 0x2i32 {
                        ret |= 0x4i32
                    }
                }
            }
        }
    }
    if 0 != qr_locked {
        pthread_mutex_unlock(&mut (*context).bobs_qr_critical);
        qr_locked = 0i32
    }
    dc_contact_unref(contact);
    free(scanned_fingerprint_of_alice as *mut libc::c_void);
    free(auth as *mut libc::c_void);
    free(own_fingerprint as *mut libc::c_void);
    free(grpid as *mut libc::c_void);
    return ret;
}
unsafe extern "C" fn end_bobs_joining(mut context: *mut dc_context_t, mut status: libc::c_int) {
    (*context).bobs_status = status;
    dc_stop_ongoing_process(context);
}
unsafe extern "C" fn secure_connection_established(
    mut context: *mut dc_context_t,
    mut contact_chat_id: uint32_t,
) {
    let mut contact_id: uint32_t = chat_id_2_contact_id(context, contact_chat_id);
    let mut contact: *mut dc_contact_t = dc_get_contact(context, contact_id);
    let mut msg: *mut libc::c_char = dc_stock_str_repl_string(
        context,
        35i32,
        if !contact.is_null() {
            (*contact).addr
        } else {
            b"?\x00" as *const u8 as *const libc::c_char
        },
    );
    dc_add_device_msg(context, contact_chat_id, msg);
    (*context).cb.expect("non-null function pointer")(
        context,
        2020i32,
        contact_chat_id as uintptr_t,
        0i32 as uintptr_t,
    );
    free(msg as *mut libc::c_void);
    dc_contact_unref(contact);
}
unsafe extern "C" fn lookup_field(
    mut mimeparser: *mut dc_mimeparser_t,
    mut key: *const libc::c_char,
) -> *const libc::c_char {
    let mut value: *const libc::c_char = 0 as *const libc::c_char;
    let mut field: *mut mailimf_field = dc_mimeparser_lookup_field(mimeparser, key);
    if field.is_null()
        || (*field).fld_type != MAILIMF_FIELD_OPTIONAL_FIELD as libc::c_int
        || (*field).fld_data.fld_optional_field.is_null()
        || {
            value = (*(*field).fld_data.fld_optional_field).fld_value;
            value.is_null()
        }
    {
        return 0 as *const libc::c_char;
    }
    return value;
}
unsafe extern "C" fn could_not_establish_secure_connection(
    mut context: *mut dc_context_t,
    mut contact_chat_id: uint32_t,
    mut details: *const libc::c_char,
) {
    let mut contact_id: uint32_t = chat_id_2_contact_id(context, contact_chat_id);
    let mut contact: *mut dc_contact_t = dc_get_contact(context, contact_id);
    let mut msg: *mut libc::c_char = dc_stock_str_repl_string(
        context,
        36i32,
        if !contact.is_null() {
            (*contact).addr
        } else {
            b"?\x00" as *const u8 as *const libc::c_char
        },
    );
    dc_add_device_msg(context, contact_chat_id, msg);
    dc_log_error(
        context,
        0i32,
        b"%s (%s)\x00" as *const u8 as *const libc::c_char,
        msg,
        details,
    );
    free(msg as *mut libc::c_void);
    dc_contact_unref(contact);
}
unsafe extern "C" fn mark_peer_as_verified(
    mut context: *mut dc_context_t,
    mut fingerprint: *const libc::c_char,
) -> libc::c_int {
    let mut success: libc::c_int = 0i32;
    let mut peerstate: *mut dc_apeerstate_t = dc_apeerstate_new(context);
    if !(0 == dc_apeerstate_load_by_fingerprint(peerstate, (*context).sql, fingerprint)) {
        if !(0 == dc_apeerstate_set_verified(peerstate, 1i32, fingerprint, 2i32)) {
            (*peerstate).prefer_encrypt = 1i32;
            (*peerstate).to_save |= 0x2i32;
            dc_apeerstate_save_to_db(peerstate, (*context).sql, 0i32);
            success = 1i32
        }
    }
    dc_apeerstate_unref(peerstate);
    return success;
}
/* ******************************************************************************
 * Tools: Misc.
 ******************************************************************************/
unsafe extern "C" fn encrypted_and_signed(
    mut mimeparser: *mut dc_mimeparser_t,
    mut expected_fingerprint: *const libc::c_char,
) -> libc::c_int {
    if 0 == (*(*mimeparser).e2ee_helper).encrypted {
        dc_log_warning(
            (*mimeparser).context,
            0i32,
            b"Message not encrypted.\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
    }
    if (*(*(*mimeparser).e2ee_helper).signatures).count <= 0i32 {
        dc_log_warning(
            (*mimeparser).context,
            0i32,
            b"Message not signed.\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
    }
    if expected_fingerprint.is_null() {
        dc_log_warning(
            (*mimeparser).context,
            0i32,
            b"Fingerprint for comparison missing.\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
    }
    if dc_hash_find(
        (*(*mimeparser).e2ee_helper).signatures,
        expected_fingerprint as *const libc::c_void,
        strlen(expected_fingerprint) as libc::c_int,
    )
    .is_null()
    {
        dc_log_warning(
            (*mimeparser).context,
            0i32,
            b"Message does not match expected fingerprint %s.\x00" as *const u8
                as *const libc::c_char,
            expected_fingerprint,
        );
        return 0i32;
    }
    return 1i32;
}
#[no_mangle]
pub unsafe extern "C" fn dc_handle_degrade_event(
    mut context: *mut dc_context_t,
    mut peerstate: *mut dc_apeerstate_t,
) {
    let mut stmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut contact_id: uint32_t = 0i32 as uint32_t;
    let mut contact_chat_id: uint32_t = 0i32 as uint32_t;
    if !(context.is_null() || peerstate.is_null()) {
        // - we do not issue an warning for DC_DE_ENCRYPTION_PAUSED as this is quite normal
        // - currently, we do not issue an extra warning for DC_DE_VERIFICATION_LOST - this always comes
        //   together with DC_DE_FINGERPRINT_CHANGED which is logged, the idea is not to bother
        //   with things they cannot fix, so the user is just kicked from the verified group
        //   (and he will know this and can fix this)
        if 0 != (*peerstate).degrade_event & 0x2i32 {
            stmt = dc_sqlite3_prepare(
                (*context).sql,
                b"SELECT id FROM contacts WHERE addr=?;\x00" as *const u8 as *const libc::c_char,
            );
            sqlite3_bind_text(stmt, 1i32, (*peerstate).addr, -1i32, None);
            sqlite3_step(stmt);
            contact_id = sqlite3_column_int(stmt, 0i32) as uint32_t;
            sqlite3_finalize(stmt);
            if !(contact_id == 0i32 as libc::c_uint) {
                dc_create_or_lookup_nchat_by_contact_id(
                    context,
                    contact_id,
                    2i32,
                    &mut contact_chat_id,
                    0 as *mut libc::c_int,
                );
                let mut msg: *mut libc::c_char =
                    dc_stock_str_repl_string(context, 37i32, (*peerstate).addr);
                dc_add_device_msg(context, contact_chat_id, msg);
                free(msg as *mut libc::c_void);
                (*context).cb.expect("non-null function pointer")(
                    context,
                    2020i32,
                    contact_chat_id as uintptr_t,
                    0i32 as uintptr_t,
                );
            }
        }
    };
}