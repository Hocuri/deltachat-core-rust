use c2rust_bitfields::BitfieldStruct;
use libc;

use crate::dc_contact::*;
use crate::dc_context::dc_context_t;
use crate::dc_e2ee::*;
use crate::dc_hash::*;
use crate::dc_location::*;
use crate::dc_log::*;
use crate::dc_lot::dc_lot_t;
use crate::dc_param::*;
use crate::dc_simplify::*;
use crate::dc_sqlite3::*;
use crate::dc_stock::*;
use crate::dc_strbuilder::*;
use crate::dc_strencode::*;
use crate::dc_tools::*;
use crate::types::*;
use crate::x::*;

/* Parse MIME body; this is the text part of an IMF, see https://tools.ietf.org/html/rfc5322
dc_mimeparser_t has no deep dependencies to dc_context_t or to the database
(dc_context_t is used for logging only). */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dc_mimepart_t {
    pub type_0: libc::c_int,
    pub is_meta: libc::c_int,
    pub int_mimetype: libc::c_int,
    pub msg: *mut libc::c_char,
    pub msg_raw: *mut libc::c_char,
    pub bytes: libc::c_int,
    pub param: *mut dc_param_t,
}

/* *
 * @class dc_mimeparser_t
 */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dc_mimeparser_t {
    pub parts: *mut carray,
    pub mimeroot: *mut mailmime,
    pub header: dc_hash_t,
    pub header_root: *mut mailimf_fields,
    pub header_protected: *mut mailimf_fields,
    pub subject: *mut libc::c_char,
    pub is_send_by_messenger: libc::c_int,
    pub decrypting_failed: libc::c_int,
    pub e2ee_helper: *mut dc_e2ee_helper_t,
    pub blobdir: *const libc::c_char,
    pub is_forwarded: libc::c_int,
    pub context: *mut dc_context_t,
    pub reports: *mut carray,
    pub is_system_message: libc::c_int,
    pub kml: *mut dc_kml_t,
}

// deprecated
#[no_mangle]
pub unsafe extern "C" fn dc_no_compound_msgs() {
    s_generate_compound_msgs = 0i32;
}
// deprecated: flag to switch generation of compound messages on and off.
static mut s_generate_compound_msgs: libc::c_int = 1i32;
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_new(
    mut blobdir: *const libc::c_char,
    mut context: *mut dc_context_t,
) -> *mut dc_mimeparser_t {
    let mut mimeparser: *mut dc_mimeparser_t = 0 as *mut dc_mimeparser_t;
    mimeparser = calloc(
        1i32 as libc::c_ulong,
        ::std::mem::size_of::<dc_mimeparser_t>() as libc::c_ulong,
    ) as *mut dc_mimeparser_t;
    if mimeparser.is_null() {
        exit(30i32);
    }
    (*mimeparser).context = context;
    (*mimeparser).parts = carray_new(16i32 as libc::c_uint);
    (*mimeparser).blobdir = blobdir;
    (*mimeparser).reports = carray_new(16i32 as libc::c_uint);
    (*mimeparser).e2ee_helper = calloc(
        1i32 as libc::c_ulong,
        ::std::mem::size_of::<dc_e2ee_helper_t>() as libc::c_ulong,
    ) as *mut dc_e2ee_helper_t;
    dc_hash_init(&mut (*mimeparser).header, 3i32, 0i32);
    return mimeparser;
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_unref(mut mimeparser: *mut dc_mimeparser_t) {
    if mimeparser.is_null() {
        return;
    }
    dc_mimeparser_empty(mimeparser);
    if !(*mimeparser).parts.is_null() {
        carray_free((*mimeparser).parts);
    }
    if !(*mimeparser).reports.is_null() {
        carray_free((*mimeparser).reports);
    }
    free((*mimeparser).e2ee_helper as *mut libc::c_void);
    free(mimeparser as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_empty(mut mimeparser: *mut dc_mimeparser_t) {
    if mimeparser.is_null() {
        return;
    }
    if !(*mimeparser).parts.is_null() {
        let mut i: libc::c_int = 0;
        let mut cnt: libc::c_int = carray_count((*mimeparser).parts) as libc::c_int;
        i = 0i32;
        while i < cnt {
            let mut part: *mut dc_mimepart_t =
                carray_get((*mimeparser).parts, i as libc::c_uint) as *mut dc_mimepart_t;
            if !part.is_null() {
                dc_mimepart_unref(part);
            }
            i += 1
        }
        carray_set_size((*mimeparser).parts, 0i32 as libc::c_uint);
    }
    (*mimeparser).header_root = 0 as *mut mailimf_fields;
    dc_hash_clear(&mut (*mimeparser).header);
    if !(*mimeparser).header_protected.is_null() {
        mailimf_fields_free((*mimeparser).header_protected);
        (*mimeparser).header_protected = 0 as *mut mailimf_fields
    }
    (*mimeparser).is_send_by_messenger = 0i32;
    (*mimeparser).is_system_message = 0i32;
    free((*mimeparser).subject as *mut libc::c_void);
    (*mimeparser).subject = 0 as *mut libc::c_char;
    if !(*mimeparser).mimeroot.is_null() {
        mailmime_free((*mimeparser).mimeroot);
        (*mimeparser).mimeroot = 0 as *mut mailmime
    }
    (*mimeparser).is_forwarded = 0i32;
    if !(*mimeparser).reports.is_null() {
        carray_set_size((*mimeparser).reports, 0i32 as libc::c_uint);
    }
    (*mimeparser).decrypting_failed = 0i32;
    dc_e2ee_thanks((*mimeparser).e2ee_helper);
    dc_kml_unref((*mimeparser).kml);
    (*mimeparser).kml = 0 as *mut dc_kml_t;
}
unsafe extern "C" fn dc_mimepart_unref(mut mimepart: *mut dc_mimepart_t) {
    if mimepart.is_null() {
        return;
    }
    free((*mimepart).msg as *mut libc::c_void);
    (*mimepart).msg = 0 as *mut libc::c_char;
    free((*mimepart).msg_raw as *mut libc::c_void);
    (*mimepart).msg_raw = 0 as *mut libc::c_char;
    dc_param_unref((*mimepart).param);
    free(mimepart as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_parse(
    mut mimeparser: *mut dc_mimeparser_t,
    mut body_not_terminated: *const libc::c_char,
    mut body_bytes: size_t,
) {
    let mut r: libc::c_int = 0i32;
    let mut index: size_t = 0i32 as size_t;
    let mut optional_field: *mut mailimf_optional_field = 0 as *mut mailimf_optional_field;
    dc_mimeparser_empty(mimeparser);
    r = mailmime_parse(
        body_not_terminated,
        body_bytes,
        &mut index,
        &mut (*mimeparser).mimeroot,
    );
    if !(r != MAILIMF_NO_ERROR as libc::c_int || (*mimeparser).mimeroot.is_null()) {
        dc_e2ee_decrypt(
            (*mimeparser).context,
            (*mimeparser).mimeroot,
            (*mimeparser).e2ee_helper,
        );
        dc_mimeparser_parse_mime_recursive(mimeparser, (*mimeparser).mimeroot);
        let mut field: *mut mailimf_field = dc_mimeparser_lookup_field(
            mimeparser,
            b"Subject\x00" as *const u8 as *const libc::c_char,
        );
        if !field.is_null() && (*field).fld_type == MAILIMF_FIELD_SUBJECT as libc::c_int {
            (*mimeparser).subject =
                dc_decode_header_words((*(*field).fld_data.fld_subject).sbj_value)
        }
        if !dc_mimeparser_lookup_optional_field(
            mimeparser,
            b"Chat-Version\x00" as *const u8 as *const libc::c_char,
        )
        .is_null()
        {
            (*mimeparser).is_send_by_messenger = 1i32
        }
        if !dc_mimeparser_lookup_field(
            mimeparser,
            b"Autocrypt-Setup-Message\x00" as *const u8 as *const libc::c_char,
        )
        .is_null()
        {
            let mut i: libc::c_int = 0;
            let mut has_setup_file: libc::c_int = 0i32;
            i = 0i32;
            while (i as libc::c_uint) < carray_count((*mimeparser).parts) {
                let mut part: *mut dc_mimepart_t =
                    carray_get((*mimeparser).parts, i as libc::c_uint) as *mut dc_mimepart_t;
                if (*part).int_mimetype == 111i32 {
                    has_setup_file = 1i32
                }
                i += 1
            }
            if 0 != has_setup_file {
                (*mimeparser).is_system_message = 6i32;
                i = 0i32;
                while (i as libc::c_uint) < carray_count((*mimeparser).parts) {
                    let mut part_0: *mut dc_mimepart_t =
                        carray_get((*mimeparser).parts, i as libc::c_uint) as *mut dc_mimepart_t;
                    if (*part_0).int_mimetype != 111i32 {
                        dc_mimepart_unref(part_0);
                        carray_delete_slow((*mimeparser).parts, i as libc::c_uint);
                        i -= 1
                    }
                    i += 1
                }
            }
        } else {
            optional_field = dc_mimeparser_lookup_optional_field(
                mimeparser,
                b"Chat-Content\x00" as *const u8 as *const libc::c_char,
            );
            if !optional_field.is_null() && !(*optional_field).fld_value.is_null() {
                if strcmp(
                    (*optional_field).fld_value,
                    b"location-streaming-enabled\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    (*mimeparser).is_system_message = 8i32
                }
            }
        }
        if !dc_mimeparser_lookup_field(
            mimeparser,
            b"Chat-Group-Image\x00" as *const u8 as *const libc::c_char,
        )
        .is_null()
            && carray_count((*mimeparser).parts) >= 1i32 as libc::c_uint
        {
            let mut textpart: *mut dc_mimepart_t =
                carray_get((*mimeparser).parts, 0i32 as libc::c_uint) as *mut dc_mimepart_t;
            if (*textpart).type_0 == 10i32 {
                if carray_count((*mimeparser).parts) >= 2i32 as libc::c_uint {
                    let mut imgpart: *mut dc_mimepart_t =
                        carray_get((*mimeparser).parts, 1i32 as libc::c_uint) as *mut dc_mimepart_t;
                    if (*imgpart).type_0 == 20i32 {
                        (*imgpart).is_meta = 1i32
                    }
                }
            }
        }
        if 0 != (*mimeparser).is_send_by_messenger
            && 0 != s_generate_compound_msgs
            && carray_count((*mimeparser).parts) == 2i32 as libc::c_uint
        {
            let mut textpart_0: *mut dc_mimepart_t =
                carray_get((*mimeparser).parts, 0i32 as libc::c_uint) as *mut dc_mimepart_t;
            let mut filepart: *mut dc_mimepart_t =
                carray_get((*mimeparser).parts, 1i32 as libc::c_uint) as *mut dc_mimepart_t;
            if (*textpart_0).type_0 == 10i32
                && ((*filepart).type_0 == 20i32
                    || (*filepart).type_0 == 21i32
                    || (*filepart).type_0 == 40i32
                    || (*filepart).type_0 == 41i32
                    || (*filepart).type_0 == 50i32
                    || (*filepart).type_0 == 60i32)
                && 0 == (*filepart).is_meta
            {
                free((*filepart).msg as *mut libc::c_void);
                (*filepart).msg = (*textpart_0).msg;
                (*textpart_0).msg = 0 as *mut libc::c_char;
                dc_mimepart_unref(textpart_0);
                carray_delete_slow((*mimeparser).parts, 0i32 as libc::c_uint);
            }
        }
        if !(*mimeparser).subject.is_null() {
            let mut prepend_subject: libc::c_int = 1i32;
            if 0 == (*mimeparser).decrypting_failed {
                let mut p: *mut libc::c_char = strchr((*mimeparser).subject, ':' as i32);
                if p.wrapping_offset_from((*mimeparser).subject) as libc::c_long
                    == 2i32 as libc::c_long
                    || p.wrapping_offset_from((*mimeparser).subject) as libc::c_long
                        == 3i32 as libc::c_long
                    || 0 != (*mimeparser).is_send_by_messenger
                    || !strstr(
                        (*mimeparser).subject,
                        b"Chat:\x00" as *const u8 as *const libc::c_char,
                    )
                    .is_null()
                {
                    prepend_subject = 0i32
                }
            }
            if 0 != prepend_subject {
                let mut subj: *mut libc::c_char = dc_strdup((*mimeparser).subject);
                let mut p_0: *mut libc::c_char = strchr(subj, '[' as i32);
                if !p_0.is_null() {
                    *p_0 = 0i32 as libc::c_char
                }
                dc_trim(subj);
                if 0 != *subj.offset(0isize) {
                    let mut i_0: libc::c_int = 0;
                    let mut icnt: libc::c_int = carray_count((*mimeparser).parts) as libc::c_int;
                    i_0 = 0i32;
                    while i_0 < icnt {
                        let mut part_1: *mut dc_mimepart_t =
                            carray_get((*mimeparser).parts, i_0 as libc::c_uint)
                                as *mut dc_mimepart_t;
                        if (*part_1).type_0 == 10i32 {
                            let mut new_txt: *mut libc::c_char = dc_mprintf(
                                b"%s \xe2\x80\x93 %s\x00" as *const u8 as *const libc::c_char,
                                subj,
                                (*part_1).msg,
                            );
                            free((*part_1).msg as *mut libc::c_void);
                            (*part_1).msg = new_txt;
                            break;
                        } else {
                            i_0 += 1
                        }
                    }
                }
                free(subj as *mut libc::c_void);
            }
        }
        if 0 != (*mimeparser).is_forwarded {
            let mut i_1: libc::c_int = 0;
            let mut icnt_0: libc::c_int = carray_count((*mimeparser).parts) as libc::c_int;
            i_1 = 0i32;
            while i_1 < icnt_0 {
                let mut part_2: *mut dc_mimepart_t =
                    carray_get((*mimeparser).parts, i_1 as libc::c_uint) as *mut dc_mimepart_t;
                dc_param_set_int((*part_2).param, 'a' as i32, 1i32);
                i_1 += 1
            }
        }
        if carray_count((*mimeparser).parts) == 1i32 as libc::c_uint {
            let mut part_3: *mut dc_mimepart_t =
                carray_get((*mimeparser).parts, 0i32 as libc::c_uint) as *mut dc_mimepart_t;
            if (*part_3).type_0 == 40i32 {
                if !dc_mimeparser_lookup_optional_field(
                    mimeparser,
                    b"Chat-Voice-Message\x00" as *const u8 as *const libc::c_char,
                )
                .is_null()
                {
                    (*part_3).type_0 = 41i32
                }
            }
            if (*part_3).type_0 == 40i32 || (*part_3).type_0 == 41i32 || (*part_3).type_0 == 50i32 {
                let mut field_0: *const mailimf_optional_field =
                    dc_mimeparser_lookup_optional_field(
                        mimeparser,
                        b"Chat-Duration\x00" as *const u8 as *const libc::c_char,
                    );
                if !field_0.is_null() {
                    let mut duration_ms: libc::c_int = atoi((*field_0).fld_value);
                    if duration_ms > 0i32 && duration_ms < 24i32 * 60i32 * 60i32 * 1000i32 {
                        dc_param_set_int((*part_3).param, 'd' as i32, duration_ms);
                    }
                }
            }
        }
        if 0 == (*mimeparser).decrypting_failed {
            let mut dn_field: *const mailimf_optional_field = dc_mimeparser_lookup_optional_field(
                mimeparser,
                b"Chat-Disposition-Notification-To\x00" as *const u8 as *const libc::c_char,
            );
            if !dn_field.is_null() && !dc_mimeparser_get_last_nonmeta(mimeparser).is_null() {
                let mut mb_list: *mut mailimf_mailbox_list = 0 as *mut mailimf_mailbox_list;
                let mut index_0: size_t = 0i32 as size_t;
                if mailimf_mailbox_list_parse(
                    (*dn_field).fld_value,
                    strlen((*dn_field).fld_value),
                    &mut index_0,
                    &mut mb_list,
                ) == MAILIMF_NO_ERROR as libc::c_int
                    && !mb_list.is_null()
                {
                    let mut dn_to_addr: *mut libc::c_char = mailimf_find_first_addr(mb_list);
                    if !dn_to_addr.is_null() {
                        let mut from_field: *mut mailimf_field = dc_mimeparser_lookup_field(
                            mimeparser,
                            b"From\x00" as *const u8 as *const libc::c_char,
                        );
                        if !from_field.is_null()
                            && (*from_field).fld_type == MAILIMF_FIELD_FROM as libc::c_int
                            && !(*from_field).fld_data.fld_from.is_null()
                        {
                            let mut from_addr: *mut libc::c_char = mailimf_find_first_addr(
                                (*(*from_field).fld_data.fld_from).frm_mb_list,
                            );
                            if !from_addr.is_null() {
                                if strcmp(from_addr, dn_to_addr) == 0i32 {
                                    let mut part_4: *mut dc_mimepart_t =
                                        dc_mimeparser_get_last_nonmeta(mimeparser);
                                    if !part_4.is_null() {
                                        dc_param_set_int((*part_4).param, 'r' as i32, 1i32);
                                    }
                                }
                                free(from_addr as *mut libc::c_void);
                            }
                        }
                        free(dn_to_addr as *mut libc::c_void);
                    }
                    mailimf_mailbox_list_free(mb_list);
                }
            }
        }
    }
    /* Cleanup - and try to create at least an empty part if there are no parts yet */
    if dc_mimeparser_get_last_nonmeta(mimeparser).is_null()
        && carray_count((*mimeparser).reports) == 0i32 as libc::c_uint
    {
        let mut part_5: *mut dc_mimepart_t = dc_mimepart_new();
        (*part_5).type_0 = 10i32;
        if !(*mimeparser).subject.is_null() && 0 == (*mimeparser).is_send_by_messenger {
            (*part_5).msg = dc_strdup((*mimeparser).subject)
        } else {
            (*part_5).msg = dc_strdup(b"\x00" as *const u8 as *const libc::c_char)
        }
        carray_add(
            (*mimeparser).parts,
            part_5 as *mut libc::c_void,
            0 as *mut libc::c_uint,
        );
    };
}
/* ******************************************************************************
 * a MIME part
 ******************************************************************************/
unsafe extern "C" fn dc_mimepart_new() -> *mut dc_mimepart_t {
    let mut mimepart: *mut dc_mimepart_t = 0 as *mut dc_mimepart_t;
    mimepart = calloc(
        1i32 as libc::c_ulong,
        ::std::mem::size_of::<dc_mimepart_t>() as libc::c_ulong,
    ) as *mut dc_mimepart_t;
    if mimepart.is_null() {
        exit(33i32);
    }
    (*mimepart).type_0 = 0i32;
    (*mimepart).param = dc_param_new();
    return mimepart;
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_get_last_nonmeta(
    mut mimeparser: *mut dc_mimeparser_t,
) -> *mut dc_mimepart_t {
    if !mimeparser.is_null() && !(*mimeparser).parts.is_null() {
        let mut i: libc::c_int = 0;
        let mut icnt: libc::c_int = carray_count((*mimeparser).parts) as libc::c_int;
        i = icnt - 1i32;
        while i >= 0i32 {
            let mut part: *mut dc_mimepart_t =
                carray_get((*mimeparser).parts, i as libc::c_uint) as *mut dc_mimepart_t;
            if !part.is_null() && 0 == (*part).is_meta {
                return part;
            }
            i -= 1
        }
    }
    return 0 as *mut dc_mimepart_t;
}
/*the result must be freed*/
#[no_mangle]
pub unsafe extern "C" fn mailimf_find_first_addr(
    mut mb_list: *const mailimf_mailbox_list,
) -> *mut libc::c_char {
    if mb_list.is_null() {
        return 0 as *mut libc::c_char;
    }
    let mut cur: *mut clistiter = (*(*mb_list).mb_list).first;
    while !cur.is_null() {
        let mut mb: *mut mailimf_mailbox = (if !cur.is_null() {
            (*cur).data
        } else {
            0 as *mut libc::c_void
        }) as *mut mailimf_mailbox;
        if !mb.is_null() && !(*mb).mb_addr_spec.is_null() {
            return dc_addr_normalize((*mb).mb_addr_spec);
        }
        cur = if !cur.is_null() {
            (*cur).next
        } else {
            0 as *mut clistcell_s
        }
    }
    return 0 as *mut libc::c_char;
}
/* the following functions can be used only after a call to dc_mimeparser_parse() */
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_lookup_field(
    mut mimeparser: *mut dc_mimeparser_t,
    mut field_name: *const libc::c_char,
) -> *mut mailimf_field {
    return dc_hash_find(
        &mut (*mimeparser).header,
        field_name as *const libc::c_void,
        strlen(field_name) as libc::c_int,
    ) as *mut mailimf_field;
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_lookup_optional_field(
    mut mimeparser: *mut dc_mimeparser_t,
    mut field_name: *const libc::c_char,
) -> *mut mailimf_optional_field {
    let mut field: *mut mailimf_field = dc_hash_find(
        &mut (*mimeparser).header,
        field_name as *const libc::c_void,
        strlen(field_name) as libc::c_int,
    ) as *mut mailimf_field;
    if !field.is_null() && (*field).fld_type == MAILIMF_FIELD_OPTIONAL_FIELD as libc::c_int {
        return (*field).fld_data.fld_optional_field;
    }
    return 0 as *mut mailimf_optional_field;
}
unsafe extern "C" fn dc_mimeparser_parse_mime_recursive(
    mut mimeparser: *mut dc_mimeparser_t,
    mut mime: *mut mailmime,
) -> libc::c_int {
    let mut any_part_added: libc::c_int = 0i32;
    let mut cur: *mut clistiter = 0 as *mut clistiter;
    if mimeparser.is_null() || mime.is_null() {
        return 0i32;
    }
    if !mailmime_find_ct_parameter(
        mime,
        b"protected-headers\x00" as *const u8 as *const libc::c_char,
    )
    .is_null()
    {
        if (*mime).mm_type == MAILMIME_SINGLE as libc::c_int
            && (*(*(*mime).mm_content_type).ct_type).tp_type
                == MAILMIME_TYPE_DISCRETE_TYPE as libc::c_int
            && (*(*(*(*mime).mm_content_type).ct_type)
                .tp_data
                .tp_discrete_type)
                .dt_type
                == MAILMIME_DISCRETE_TYPE_TEXT as libc::c_int
            && !(*(*mime).mm_content_type).ct_subtype.is_null()
            && strcmp(
                (*(*mime).mm_content_type).ct_subtype,
                b"rfc822-headers\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
        {
            dc_log_info(
                (*mimeparser).context,
                0i32,
                b"Protected headers found in text/rfc822-headers attachment: Will be ignored.\x00"
                    as *const u8 as *const libc::c_char,
            );
            return 0i32;
        }
        if (*mimeparser).header_protected.is_null() {
            let mut dummy: size_t = 0i32 as size_t;
            if mailimf_envelope_and_optional_fields_parse(
                (*mime).mm_mime_start,
                (*mime).mm_length,
                &mut dummy,
                &mut (*mimeparser).header_protected,
            ) != MAILIMF_NO_ERROR as libc::c_int
                || (*mimeparser).header_protected.is_null()
            {
                dc_log_warning(
                    (*mimeparser).context,
                    0i32,
                    b"Protected headers parsing error.\x00" as *const u8 as *const libc::c_char,
                );
            } else {
                hash_header(
                    &mut (*mimeparser).header,
                    (*mimeparser).header_protected,
                    (*mimeparser).context,
                );
            }
        } else {
            dc_log_info((*mimeparser).context, 0i32,
                        b"Protected headers found in MIME header: Will be ignored as we already found an outer one.\x00"
                            as *const u8 as *const libc::c_char);
        }
    }
    match (*mime).mm_type {
        1 => any_part_added = dc_mimeparser_add_single_part_if_known(mimeparser, mime),
        2 => {
            match mailmime_get_mime_type(mime, 0 as *mut libc::c_int, 0 as *mut *mut libc::c_char) {
                10 => {
                    cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                    while !cur.is_null() {
                        let mut childmime: *mut mailmime = (if !cur.is_null() {
                            (*cur).data
                        } else {
                            0 as *mut libc::c_void
                        })
                            as *mut mailmime;
                        if mailmime_get_mime_type(
                            childmime,
                            0 as *mut libc::c_int,
                            0 as *mut *mut libc::c_char,
                        ) == 30i32
                        {
                            any_part_added =
                                dc_mimeparser_parse_mime_recursive(mimeparser, childmime);
                            break;
                        } else {
                            cur = if !cur.is_null() {
                                (*cur).next
                            } else {
                                0 as *mut clistcell_s
                            }
                        }
                    }
                    if 0 == any_part_added {
                        cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                        while !cur.is_null() {
                            let mut childmime_0: *mut mailmime = (if !cur.is_null() {
                                (*cur).data
                            } else {
                                0 as *mut libc::c_void
                            })
                                as *mut mailmime;
                            if mailmime_get_mime_type(
                                childmime_0,
                                0 as *mut libc::c_int,
                                0 as *mut *mut libc::c_char,
                            ) == 60i32
                            {
                                any_part_added =
                                    dc_mimeparser_parse_mime_recursive(mimeparser, childmime_0);
                                break;
                            } else {
                                cur = if !cur.is_null() {
                                    (*cur).next
                                } else {
                                    0 as *mut clistcell_s
                                }
                            }
                        }
                    }
                    if 0 == any_part_added {
                        cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                        while !cur.is_null() {
                            if 0 != dc_mimeparser_parse_mime_recursive(
                                mimeparser,
                                (if !cur.is_null() {
                                    (*cur).data
                                } else {
                                    0 as *mut libc::c_void
                                }) as *mut mailmime,
                            ) {
                                any_part_added = 1i32;
                                /* out of for() */
                                break;
                            } else {
                                cur = if !cur.is_null() {
                                    (*cur).next
                                } else {
                                    0 as *mut clistcell_s
                                }
                            }
                        }
                    }
                }
                20 => {
                    cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                    if !cur.is_null() {
                        any_part_added = dc_mimeparser_parse_mime_recursive(
                            mimeparser,
                            (if !cur.is_null() {
                                (*cur).data
                            } else {
                                0 as *mut libc::c_void
                            }) as *mut mailmime,
                        )
                    }
                }
                40 => {
                    let mut part: *mut dc_mimepart_t = dc_mimepart_new();
                    (*part).type_0 = 10i32;
                    let mut msg_body: *mut libc::c_char =
                        dc_stock_str((*mimeparser).context, 29i32);
                    (*part).msg =
                        dc_mprintf(b"[%s]\x00" as *const u8 as *const libc::c_char, msg_body);
                    (*part).msg_raw = dc_strdup((*part).msg);
                    free(msg_body as *mut libc::c_void);
                    carray_add(
                        (*mimeparser).parts,
                        part as *mut libc::c_void,
                        0 as *mut libc::c_uint,
                    );
                    any_part_added = 1i32;
                    (*mimeparser).decrypting_failed = 1i32
                }
                46 => {
                    cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                    if !cur.is_null() {
                        any_part_added = dc_mimeparser_parse_mime_recursive(
                            mimeparser,
                            (if !cur.is_null() {
                                (*cur).data
                            } else {
                                0 as *mut libc::c_void
                            }) as *mut mailmime,
                        )
                    }
                }
                45 => {
                    if (*(*mime).mm_data.mm_multipart.mm_mp_list).count >= 2i32 {
                        let mut report_type: *mut mailmime_parameter = mailmime_find_ct_parameter(
                            mime,
                            b"report-type\x00" as *const u8 as *const libc::c_char,
                        );
                        if !report_type.is_null()
                            && !(*report_type).pa_value.is_null()
                            && strcmp(
                                (*report_type).pa_value,
                                b"disposition-notification\x00" as *const u8 as *const libc::c_char,
                            ) == 0i32
                        {
                            carray_add(
                                (*mimeparser).reports,
                                mime as *mut libc::c_void,
                                0 as *mut libc::c_uint,
                            );
                        } else {
                            any_part_added = dc_mimeparser_parse_mime_recursive(
                                mimeparser,
                                (if !(*(*mime).mm_data.mm_multipart.mm_mp_list).first.is_null() {
                                    (*(*(*mime).mm_data.mm_multipart.mm_mp_list).first).data
                                } else {
                                    0 as *mut libc::c_void
                                }) as *mut mailmime,
                            )
                        }
                    }
                }
                _ => {
                    let mut skip_part: *mut mailmime = 0 as *mut mailmime;
                    let mut html_part: *mut mailmime = 0 as *mut mailmime;
                    let mut plain_cnt: libc::c_int = 0i32;
                    let mut html_cnt: libc::c_int = 0i32;
                    cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                    while !cur.is_null() {
                        let mut childmime_1: *mut mailmime = (if !cur.is_null() {
                            (*cur).data
                        } else {
                            0 as *mut libc::c_void
                        })
                            as *mut mailmime;
                        if mailmime_get_mime_type(
                            childmime_1,
                            0 as *mut libc::c_int,
                            0 as *mut *mut libc::c_char,
                        ) == 60i32
                        {
                            plain_cnt += 1
                        } else if mailmime_get_mime_type(
                            childmime_1,
                            0 as *mut libc::c_int,
                            0 as *mut *mut libc::c_char,
                        ) == 70i32
                        {
                            html_part = childmime_1;
                            html_cnt += 1
                        }
                        cur = if !cur.is_null() {
                            (*cur).next
                        } else {
                            0 as *mut clistcell_s
                        }
                    }
                    if plain_cnt == 1i32 && html_cnt == 1i32 {
                        dc_log_warning((*mimeparser).context, 0i32,
                                       b"HACK: multipart/mixed message found with PLAIN and HTML, we\'ll skip the HTML part as this seems to be unwanted.\x00"
                                           as *const u8 as
                                           *const libc::c_char);
                        skip_part = html_part
                    }
                    cur = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
                    while !cur.is_null() {
                        let mut childmime_2: *mut mailmime = (if !cur.is_null() {
                            (*cur).data
                        } else {
                            0 as *mut libc::c_void
                        })
                            as *mut mailmime;
                        if childmime_2 != skip_part {
                            if 0 != dc_mimeparser_parse_mime_recursive(mimeparser, childmime_2) {
                                any_part_added = 1i32
                            }
                        }
                        cur = if !cur.is_null() {
                            (*cur).next
                        } else {
                            0 as *mut clistcell_s
                        }
                    }
                }
            }
        }
        3 => {
            if (*mimeparser).header_root.is_null() {
                (*mimeparser).header_root = (*mime).mm_data.mm_message.mm_fields;
                hash_header(
                    &mut (*mimeparser).header,
                    (*mimeparser).header_root,
                    (*mimeparser).context,
                );
            }
            if !(*mime).mm_data.mm_message.mm_msg_mime.is_null() {
                any_part_added = dc_mimeparser_parse_mime_recursive(
                    mimeparser,
                    (*mime).mm_data.mm_message.mm_msg_mime,
                )
            }
        }
        _ => {}
    }
    return any_part_added;
}
unsafe extern "C" fn hash_header(
    mut out: *mut dc_hash_t,
    mut in_0: *const mailimf_fields,
    mut context: *mut dc_context_t,
) {
    if in_0.is_null() {
        return;
    }
    let mut cur1: *mut clistiter = (*(*in_0).fld_list).first;
    while !cur1.is_null() {
        let mut field: *mut mailimf_field = (if !cur1.is_null() {
            (*cur1).data
        } else {
            0 as *mut libc::c_void
        }) as *mut mailimf_field;
        let mut key: *const libc::c_char = 0 as *const libc::c_char;
        match (*field).fld_type {
            1 => key = b"Return-Path\x00" as *const u8 as *const libc::c_char,
            9 => key = b"Date\x00" as *const u8 as *const libc::c_char,
            10 => key = b"From\x00" as *const u8 as *const libc::c_char,
            11 => key = b"Sender\x00" as *const u8 as *const libc::c_char,
            12 => key = b"Reply-To\x00" as *const u8 as *const libc::c_char,
            13 => key = b"To\x00" as *const u8 as *const libc::c_char,
            14 => key = b"Cc\x00" as *const u8 as *const libc::c_char,
            15 => key = b"Bcc\x00" as *const u8 as *const libc::c_char,
            16 => key = b"Message-ID\x00" as *const u8 as *const libc::c_char,
            17 => key = b"In-Reply-To\x00" as *const u8 as *const libc::c_char,
            18 => key = b"References\x00" as *const u8 as *const libc::c_char,
            19 => key = b"Subject\x00" as *const u8 as *const libc::c_char,
            22 => {
                let mut optional_field: *const mailimf_optional_field =
                    (*field).fld_data.fld_optional_field;
                if !optional_field.is_null() {
                    key = (*optional_field).fld_name
                }
            }
            _ => {}
        }
        if !key.is_null() {
            let mut key_len: libc::c_int = strlen(key) as libc::c_int;
            if !dc_hash_find(out, key as *const libc::c_void, key_len).is_null() {
                if (*field).fld_type != MAILIMF_FIELD_OPTIONAL_FIELD as libc::c_int
                    || key_len > 5i32
                        && strncasecmp(
                            key,
                            b"Chat-\x00" as *const u8 as *const libc::c_char,
                            5i32 as libc::c_ulong,
                        ) == 0i32
                {
                    dc_hash_insert(
                        out,
                        key as *const libc::c_void,
                        key_len,
                        field as *mut libc::c_void,
                    );
                }
            } else {
                dc_hash_insert(
                    out,
                    key as *const libc::c_void,
                    key_len,
                    field as *mut libc::c_void,
                );
            }
        }
        cur1 = if !cur1.is_null() {
            (*cur1).next
        } else {
            0 as *mut clistcell_s
        }
    }
}
unsafe extern "C" fn mailmime_get_mime_type(
    mut mime: *mut mailmime,
    mut msg_type: *mut libc::c_int,
    mut raw_mime: *mut *mut libc::c_char,
) -> libc::c_int {
    let mut c: *mut mailmime_content = (*mime).mm_content_type;
    let mut dummy: libc::c_int = 0i32;
    if msg_type.is_null() {
        msg_type = &mut dummy
    }
    *msg_type = 0i32;
    if c.is_null() || (*c).ct_type.is_null() {
        return 0i32;
    }
    match (*(*c).ct_type).tp_type {
        1 => match (*(*(*c).ct_type).tp_data.tp_discrete_type).dt_type {
            1 => {
                if !(0 != mailmime_is_attachment_disposition(mime)) {
                    if strcmp(
                        (*c).ct_subtype,
                        b"plain\x00" as *const u8 as *const libc::c_char,
                    ) == 0i32
                    {
                        *msg_type = 10i32;
                        return 60i32;
                    } else {
                        if strcmp(
                            (*c).ct_subtype,
                            b"html\x00" as *const u8 as *const libc::c_char,
                        ) == 0i32
                        {
                            *msg_type = 10i32;
                            return 70i32;
                        }
                    }
                }
                *msg_type = 60i32;
                reconcat_mime(
                    raw_mime,
                    b"text\x00" as *const u8 as *const libc::c_char,
                    (*c).ct_subtype,
                );
                return 110i32;
            }
            2 => {
                if strcmp(
                    (*c).ct_subtype,
                    b"gif\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    *msg_type = 21i32
                } else if strcmp(
                    (*c).ct_subtype,
                    b"svg+xml\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    *msg_type = 60i32;
                    reconcat_mime(
                        raw_mime,
                        b"image\x00" as *const u8 as *const libc::c_char,
                        (*c).ct_subtype,
                    );
                    return 110i32;
                } else {
                    *msg_type = 20i32
                }
                reconcat_mime(
                    raw_mime,
                    b"image\x00" as *const u8 as *const libc::c_char,
                    (*c).ct_subtype,
                );
                return 80i32;
            }
            3 => {
                *msg_type = 40i32;
                reconcat_mime(
                    raw_mime,
                    b"audio\x00" as *const u8 as *const libc::c_char,
                    (*c).ct_subtype,
                );
                return 90i32;
            }
            4 => {
                *msg_type = 50i32;
                reconcat_mime(
                    raw_mime,
                    b"video\x00" as *const u8 as *const libc::c_char,
                    (*c).ct_subtype,
                );
                return 100i32;
            }
            _ => {
                *msg_type = 60i32;
                if (*(*(*c).ct_type).tp_data.tp_discrete_type).dt_type
                    == MAILMIME_DISCRETE_TYPE_APPLICATION as libc::c_int
                    && strcmp(
                        (*c).ct_subtype,
                        b"autocrypt-setup\x00" as *const u8 as *const libc::c_char,
                    ) == 0i32
                {
                    reconcat_mime(
                        raw_mime,
                        b"application\x00" as *const u8 as *const libc::c_char,
                        (*c).ct_subtype,
                    );
                    return 111i32;
                }
                reconcat_mime(
                    raw_mime,
                    (*(*(*c).ct_type).tp_data.tp_discrete_type).dt_extension,
                    (*c).ct_subtype,
                );
                return 110i32;
            }
        },
        2 => {
            if (*(*(*c).ct_type).tp_data.tp_composite_type).ct_type
                == MAILMIME_COMPOSITE_TYPE_MULTIPART as libc::c_int
            {
                if strcmp(
                    (*c).ct_subtype,
                    b"alternative\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    return 10i32;
                } else if strcmp(
                    (*c).ct_subtype,
                    b"related\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    return 20i32;
                } else if strcmp(
                    (*c).ct_subtype,
                    b"encrypted\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    return 40i32;
                } else if strcmp(
                    (*c).ct_subtype,
                    b"signed\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    return 46i32;
                } else if strcmp(
                    (*c).ct_subtype,
                    b"mixed\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    return 30i32;
                } else if strcmp(
                    (*c).ct_subtype,
                    b"report\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    return 45i32;
                } else {
                    return 50i32;
                }
            } else {
                if (*(*(*c).ct_type).tp_data.tp_composite_type).ct_type
                    == MAILMIME_COMPOSITE_TYPE_MESSAGE as libc::c_int
                {
                    return 0i32;
                }
            }
        }
        _ => {}
    }
    return 0i32;
}
unsafe extern "C" fn reconcat_mime(
    mut raw_mime: *mut *mut libc::c_char,
    mut type_0: *const libc::c_char,
    mut subtype: *const libc::c_char,
) {
    if !raw_mime.is_null() {
        *raw_mime = dc_mprintf(
            b"%s/%s\x00" as *const u8 as *const libc::c_char,
            if !type_0.is_null() {
                type_0
            } else {
                b"application\x00" as *const u8 as *const libc::c_char
            },
            if !subtype.is_null() {
                subtype
            } else {
                b"octet-stream\x00" as *const u8 as *const libc::c_char
            },
        )
    };
}
unsafe extern "C" fn mailmime_is_attachment_disposition(mut mime: *mut mailmime) -> libc::c_int {
    if !(*mime).mm_mime_fields.is_null() {
        let mut cur: *mut clistiter = (*(*(*mime).mm_mime_fields).fld_list).first;
        while !cur.is_null() {
            let mut field: *mut mailmime_field = (if !cur.is_null() {
                (*cur).data
            } else {
                0 as *mut libc::c_void
            }) as *mut mailmime_field;
            if !field.is_null()
                && (*field).fld_type == MAILMIME_FIELD_DISPOSITION as libc::c_int
                && !(*field).fld_data.fld_disposition.is_null()
            {
                if !(*(*field).fld_data.fld_disposition).dsp_type.is_null()
                    && (*(*(*field).fld_data.fld_disposition).dsp_type).dsp_type
                        == MAILMIME_DISPOSITION_TYPE_ATTACHMENT as libc::c_int
                {
                    return 1i32;
                }
            }
            cur = if !cur.is_null() {
                (*cur).next
            } else {
                0 as *mut clistcell_s
            }
        }
    }
    return 0i32;
}
/* low-level-tools for working with mailmime structures directly */
#[no_mangle]
pub unsafe extern "C" fn mailmime_find_ct_parameter(
    mut mime: *mut mailmime,
    mut name: *const libc::c_char,
) -> *mut mailmime_parameter {
    if mime.is_null()
        || name.is_null()
        || (*mime).mm_content_type.is_null()
        || (*(*mime).mm_content_type).ct_parameters.is_null()
    {
        return 0 as *mut mailmime_parameter;
    }
    let mut cur: *mut clistiter = 0 as *mut clistiter;
    cur = (*(*(*mime).mm_content_type).ct_parameters).first;
    while !cur.is_null() {
        let mut param: *mut mailmime_parameter = (if !cur.is_null() {
            (*cur).data
        } else {
            0 as *mut libc::c_void
        }) as *mut mailmime_parameter;
        if !param.is_null() && !(*param).pa_name.is_null() {
            if strcmp((*param).pa_name, name) == 0i32 {
                return param;
            }
        }
        cur = if !cur.is_null() {
            (*cur).next
        } else {
            0 as *mut clistcell_s
        }
    }
    return 0 as *mut mailmime_parameter;
}
unsafe extern "C" fn dc_mimeparser_add_single_part_if_known(
    mut mimeparser: *mut dc_mimeparser_t,
    mut mime: *mut mailmime,
) -> libc::c_int {
    let mut current_block: u64;
    let mut part: *mut dc_mimepart_t = 0 as *mut dc_mimepart_t;
    let mut old_part_count: libc::c_int = carray_count((*mimeparser).parts) as libc::c_int;
    let mut mime_type: libc::c_int = 0;
    let mut mime_data: *mut mailmime_data = 0 as *mut mailmime_data;
    let mut file_suffix: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut desired_filename: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut msg_type: libc::c_int = 0i32;
    let mut raw_mime: *mut libc::c_char = 0 as *mut libc::c_char;
    /* mmap_string_unref()'d if set */
    let mut transfer_decoding_buffer: *mut libc::c_char = 0 as *mut libc::c_char;
    /* charconv_buffer_free()'d if set (just calls mmap_string_unref()) */
    let mut charset_buffer: *mut libc::c_char = 0 as *mut libc::c_char;
    /* must not be free()'d */
    let mut decoded_data: *const libc::c_char = 0 as *const libc::c_char;
    let mut decoded_data_bytes: size_t = 0i32 as size_t;
    let mut simplifier: *mut dc_simplify_t = 0 as *mut dc_simplify_t;
    if !(mime.is_null() || (*mime).mm_data.mm_single.is_null()) {
        mime_type = mailmime_get_mime_type(mime, &mut msg_type, &mut raw_mime);
        mime_data = (*mime).mm_data.mm_single;
        /* MAILMIME_DATA_FILE indicates, the data is in a file; AFAIK this is not used on parsing */
        if !((*mime_data).dt_type != MAILMIME_DATA_TEXT as libc::c_int
            || (*mime_data).dt_data.dt_text.dt_data.is_null()
            || (*mime_data).dt_data.dt_text.dt_length <= 0i32 as libc::c_ulong)
        {
            /* regard `Content-Transfer-Encoding:` */
            if !(0
                == mailmime_transfer_decode(
                    mime,
                    &mut decoded_data,
                    &mut decoded_data_bytes,
                    &mut transfer_decoding_buffer,
                ))
            {
                /* no always error - but no data */
                match mime_type {
                    60 | 70 => {
                        if simplifier.is_null() {
                            simplifier = dc_simplify_new();
                            if simplifier.is_null() {
                                current_block = 8795901732489102124;
                            } else {
                                current_block = 13797916685926291137;
                            }
                        } else {
                            current_block = 13797916685926291137;
                        }
                        match current_block {
                            8795901732489102124 => {}
                            _ => {
                                /* get from `Content-Type: text/...; charset=utf-8`; must not be free()'d */
                                let mut charset: *const libc::c_char =
                                    mailmime_content_charset_get((*mime).mm_content_type);
                                if !charset.is_null()
                                    && strcmp(
                                        charset,
                                        b"utf-8\x00" as *const u8 as *const libc::c_char,
                                    ) != 0i32
                                    && strcmp(
                                        charset,
                                        b"UTF-8\x00" as *const u8 as *const libc::c_char,
                                    ) != 0i32
                                {
                                    let mut ret_bytes: size_t = 0i32 as size_t;
                                    let mut r: libc::c_int = charconv_buffer(
                                        b"utf-8\x00" as *const u8 as *const libc::c_char,
                                        charset,
                                        decoded_data,
                                        decoded_data_bytes,
                                        &mut charset_buffer,
                                        &mut ret_bytes,
                                    );
                                    if r != MAIL_CHARCONV_NO_ERROR as libc::c_int {
                                        dc_log_warning((*mimeparser).context,
                                                       0i32,
                                                       b"Cannot convert %i bytes from \"%s\" to \"utf-8\"; errorcode is %i.\x00"
                                                           as *const u8 as
                                                           *const libc::c_char,
                                                       decoded_data_bytes as
                                                           libc::c_int,
                                                       charset,
                                                       r as libc::c_int);
                                        current_block = 17788412896529399552;
                                    } else if charset_buffer.is_null()
                                        || ret_bytes <= 0i32 as libc::c_ulong
                                    {
                                        /* no error - but nothing to add */
                                        current_block = 8795901732489102124;
                                    } else {
                                        decoded_data = charset_buffer;
                                        decoded_data_bytes = ret_bytes;
                                        current_block = 17788412896529399552;
                                    }
                                } else {
                                    current_block = 17788412896529399552;
                                }
                                match current_block {
                                    8795901732489102124 => {}
                                    _ => {
                                        /* check header directly as is_send_by_messenger is not yet set up */
                                        let mut is_msgrmsg: libc::c_int =
                                            (dc_mimeparser_lookup_optional_field(
                                                mimeparser,
                                                b"Chat-Version\x00" as *const u8
                                                    as *const libc::c_char,
                                            ) != 0 as *mut libc::c_void
                                                as *mut mailimf_optional_field)
                                                as libc::c_int;
                                        let mut simplified_txt: *mut libc::c_char =
                                            dc_simplify_simplify(
                                                simplifier,
                                                decoded_data,
                                                decoded_data_bytes as libc::c_int,
                                                if mime_type == 70i32 { 1i32 } else { 0i32 },
                                                is_msgrmsg,
                                            );
                                        if !simplified_txt.is_null()
                                            && 0 != *simplified_txt.offset(0isize) as libc::c_int
                                        {
                                            part = dc_mimepart_new();
                                            (*part).type_0 = 10i32;
                                            (*part).int_mimetype = mime_type;
                                            (*part).msg = simplified_txt;
                                            (*part).msg_raw =
                                                strndup(decoded_data, decoded_data_bytes);
                                            do_add_single_part(mimeparser, part);
                                            part = 0 as *mut dc_mimepart_t
                                        } else {
                                            free(simplified_txt as *mut libc::c_void);
                                        }
                                        if 0 != (*simplifier).is_forwarded {
                                            (*mimeparser).is_forwarded = 1i32
                                        }
                                        current_block = 10261677128829721533;
                                    }
                                }
                            }
                        }
                    }
                    80 | 90 | 100 | 110 | 111 => {
                        /* try to get file name from
                           `Content-Disposition: ... filename*=...`
                        or `Content-Disposition: ... filename*0*=... filename*1*=... filename*2*=...`
                        or `Content-Disposition: ... filename=...` */
                        let mut filename_parts: dc_strbuilder_t = dc_strbuilder_t {
                            buf: 0 as *mut libc::c_char,
                            allocated: 0,
                            free: 0,
                            eos: 0 as *mut libc::c_char,
                        };
                        dc_strbuilder_init(&mut filename_parts, 0i32);
                        let mut cur1: *mut clistiter = (*(*(*mime).mm_mime_fields).fld_list).first;
                        while !cur1.is_null() {
                            let mut field: *mut mailmime_field = (if !cur1.is_null() {
                                (*cur1).data
                            } else {
                                0 as *mut libc::c_void
                            })
                                as *mut mailmime_field;
                            if !field.is_null()
                                && (*field).fld_type == MAILMIME_FIELD_DISPOSITION as libc::c_int
                                && !(*field).fld_data.fld_disposition.is_null()
                            {
                                let mut file_disposition: *mut mailmime_disposition =
                                    (*field).fld_data.fld_disposition;
                                if !file_disposition.is_null() {
                                    let mut cur2: *mut clistiter =
                                        (*(*file_disposition).dsp_parms).first;
                                    while !cur2.is_null() {
                                        let mut dsp_param: *mut mailmime_disposition_parm =
                                            (if !cur2.is_null() {
                                                (*cur2).data
                                            } else {
                                                0 as *mut libc::c_void
                                            })
                                                as *mut mailmime_disposition_parm;
                                        if !dsp_param.is_null() {
                                            if (*dsp_param).pa_type
                                                == MAILMIME_DISPOSITION_PARM_PARAMETER
                                                    as libc::c_int
                                                && !(*dsp_param).pa_data.pa_parameter.is_null()
                                                && !(*(*dsp_param).pa_data.pa_parameter)
                                                    .pa_name
                                                    .is_null()
                                                && strncmp(
                                                    (*(*dsp_param).pa_data.pa_parameter).pa_name,
                                                    b"filename*\x00" as *const u8
                                                        as *const libc::c_char,
                                                    9i32 as libc::c_ulong,
                                                ) == 0i32
                                            {
                                                dc_strbuilder_cat(
                                                    &mut filename_parts,
                                                    (*(*dsp_param).pa_data.pa_parameter).pa_value,
                                                );
                                            } else if (*dsp_param).pa_type
                                                == MAILMIME_DISPOSITION_PARM_FILENAME as libc::c_int
                                            {
                                                desired_filename = dc_decode_header_words(
                                                    (*dsp_param).pa_data.pa_filename,
                                                )
                                            }
                                        }
                                        cur2 = if !cur2.is_null() {
                                            (*cur2).next
                                        } else {
                                            0 as *mut clistcell_s
                                        }
                                    }
                                }
                                break;
                            } else {
                                cur1 = if !cur1.is_null() {
                                    (*cur1).next
                                } else {
                                    0 as *mut clistcell_s
                                }
                            }
                        }
                        if 0 != strlen(filename_parts.buf) {
                            free(desired_filename as *mut libc::c_void);
                            desired_filename = dc_decode_ext_header(filename_parts.buf)
                        }
                        free(filename_parts.buf as *mut libc::c_void);
                        if desired_filename.is_null() {
                            let mut param: *mut mailmime_parameter = mailmime_find_ct_parameter(
                                mime,
                                b"name\x00" as *const u8 as *const libc::c_char,
                            );
                            if !param.is_null()
                                && !(*param).pa_value.is_null()
                                && 0 != *(*param).pa_value.offset(0isize) as libc::c_int
                            {
                                desired_filename = dc_strdup((*param).pa_value)
                            }
                        }
                        /* if there is still no filename, guess one */
                        if desired_filename.is_null() {
                            if !(*mime).mm_content_type.is_null()
                                && !(*(*mime).mm_content_type).ct_subtype.is_null()
                            {
                                desired_filename = dc_mprintf(
                                    b"file.%s\x00" as *const u8 as *const libc::c_char,
                                    (*(*mime).mm_content_type).ct_subtype,
                                );
                                current_block = 17019156190352891614;
                            } else {
                                current_block = 8795901732489102124;
                            }
                        } else {
                            current_block = 17019156190352891614;
                        }
                        match current_block {
                            8795901732489102124 => {}
                            _ => {
                                if strncmp(
                                    desired_filename,
                                    b"location\x00" as *const u8 as *const libc::c_char,
                                    8i32 as libc::c_ulong,
                                ) == 0i32
                                    && strncmp(
                                        desired_filename
                                            .offset(strlen(desired_filename) as isize)
                                            .offset(-4isize),
                                        b".kml\x00" as *const u8 as *const libc::c_char,
                                        4i32 as libc::c_ulong,
                                    ) == 0i32
                                {
                                    (*mimeparser).kml = dc_kml_parse(
                                        (*mimeparser).context,
                                        decoded_data,
                                        decoded_data_bytes,
                                    );
                                    current_block = 8795901732489102124;
                                } else {
                                    dc_replace_bad_utf8_chars(desired_filename);
                                    do_add_single_file_part(
                                        mimeparser,
                                        msg_type,
                                        mime_type,
                                        raw_mime,
                                        decoded_data,
                                        decoded_data_bytes,
                                        desired_filename,
                                    );
                                    current_block = 10261677128829721533;
                                }
                            }
                        }
                    }
                    _ => {
                        current_block = 10261677128829721533;
                    }
                }
                match current_block {
                    8795901732489102124 => {}
                    _ => {}
                }
            }
        }
    }
    /* add object? (we do not add all objetcs, eg. signatures etc. are ignored) */
    dc_simplify_unref(simplifier);
    if !charset_buffer.is_null() {
        charconv_buffer_free(charset_buffer);
    }
    if !transfer_decoding_buffer.is_null() {
        mmap_string_unref(transfer_decoding_buffer);
    }
    free(file_suffix as *mut libc::c_void);
    free(desired_filename as *mut libc::c_void);
    dc_mimepart_unref(part);
    free(raw_mime as *mut libc::c_void);
    return if carray_count((*mimeparser).parts) > old_part_count as libc::c_uint {
        1i32
    } else {
        0i32
    };
}
unsafe extern "C" fn do_add_single_file_part(
    mut parser: *mut dc_mimeparser_t,
    mut msg_type: libc::c_int,
    mut mime_type: libc::c_int,
    mut raw_mime: *const libc::c_char,
    mut decoded_data: *const libc::c_char,
    mut decoded_data_bytes: size_t,
    mut desired_filename: *const libc::c_char,
) {
    let mut part: *mut dc_mimepart_t = 0 as *mut dc_mimepart_t;
    let mut pathNfilename: *mut libc::c_char = 0 as *mut libc::c_char;
    /* create a free file name to use */
    pathNfilename = dc_get_fine_pathNfilename(
        (*parser).context,
        b"$BLOBDIR\x00" as *const u8 as *const libc::c_char,
        desired_filename,
    );
    if !pathNfilename.is_null() {
        /* copy data to file */
        if !(dc_write_file(
            (*parser).context,
            pathNfilename,
            decoded_data as *const libc::c_void,
            decoded_data_bytes,
        ) == 0i32)
        {
            part = dc_mimepart_new();
            (*part).type_0 = msg_type;
            (*part).int_mimetype = mime_type;
            (*part).bytes = decoded_data_bytes as libc::c_int;
            dc_param_set((*part).param, 'f' as i32, pathNfilename);
            dc_param_set((*part).param, 'm' as i32, raw_mime);
            if mime_type == 80i32 {
                let mut w: uint32_t = 0i32 as uint32_t;
                let mut h: uint32_t = 0i32 as uint32_t;
                if 0 != dc_get_filemeta(
                    decoded_data as *const libc::c_void,
                    decoded_data_bytes,
                    &mut w,
                    &mut h,
                ) {
                    dc_param_set_int((*part).param, 'w' as i32, w as int32_t);
                    dc_param_set_int((*part).param, 'h' as i32, h as int32_t);
                }
            }
            do_add_single_part(parser, part);
            part = 0 as *mut dc_mimepart_t
        }
    }
    free(pathNfilename as *mut libc::c_void);
    dc_mimepart_unref(part);
}
unsafe extern "C" fn do_add_single_part(
    mut parser: *mut dc_mimeparser_t,
    mut part: *mut dc_mimepart_t,
) {
    if 0 != (*(*parser).e2ee_helper).encrypted
        && (*(*(*parser).e2ee_helper).signatures).count > 0i32
    {
        dc_param_set_int((*part).param, 'c' as i32, 1i32);
    } else if 0 != (*(*parser).e2ee_helper).encrypted {
        dc_param_set_int((*part).param, 'e' as i32, 0x2i32);
    }
    carray_add(
        (*parser).parts,
        part as *mut libc::c_void,
        0 as *mut libc::c_uint,
    );
}
#[no_mangle]
pub unsafe extern "C" fn mailmime_transfer_decode(
    mut mime: *mut mailmime,
    mut ret_decoded_data: *mut *const libc::c_char,
    mut ret_decoded_data_bytes: *mut size_t,
    mut ret_to_mmap_string_unref: *mut *mut libc::c_char,
) -> libc::c_int {
    let mut mime_transfer_encoding: libc::c_int = MAILMIME_MECHANISM_BINARY as libc::c_int;
    let mut mime_data: *mut mailmime_data = 0 as *mut mailmime_data;
    /* must not be free()'d */
    let mut decoded_data: *const libc::c_char = 0 as *const libc::c_char;
    let mut decoded_data_bytes: size_t = 0i32 as size_t;
    /* mmap_string_unref()'d if set */
    let mut transfer_decoding_buffer: *mut libc::c_char = 0 as *mut libc::c_char;
    if mime.is_null()
        || ret_decoded_data.is_null()
        || ret_decoded_data_bytes.is_null()
        || ret_to_mmap_string_unref.is_null()
        || !(*ret_decoded_data).is_null()
        || *ret_decoded_data_bytes != 0i32 as libc::c_ulong
        || !(*ret_to_mmap_string_unref).is_null()
    {
        return 0i32;
    }
    mime_data = (*mime).mm_data.mm_single;
    if !(*mime).mm_mime_fields.is_null() {
        let mut cur: *mut clistiter = 0 as *mut clistiter;
        cur = (*(*(*mime).mm_mime_fields).fld_list).first;
        while !cur.is_null() {
            let mut field: *mut mailmime_field = (if !cur.is_null() {
                (*cur).data
            } else {
                0 as *mut libc::c_void
            }) as *mut mailmime_field;
            if !field.is_null()
                && (*field).fld_type == MAILMIME_FIELD_TRANSFER_ENCODING as libc::c_int
                && !(*field).fld_data.fld_encoding.is_null()
            {
                mime_transfer_encoding = (*(*field).fld_data.fld_encoding).enc_type;
                break;
            } else {
                cur = if !cur.is_null() {
                    (*cur).next
                } else {
                    0 as *mut clistcell_s
                }
            }
        }
    }
    if mime_transfer_encoding == MAILMIME_MECHANISM_7BIT as libc::c_int
        || mime_transfer_encoding == MAILMIME_MECHANISM_8BIT as libc::c_int
        || mime_transfer_encoding == MAILMIME_MECHANISM_BINARY as libc::c_int
    {
        decoded_data = (*mime_data).dt_data.dt_text.dt_data;
        decoded_data_bytes = (*mime_data).dt_data.dt_text.dt_length;
        if decoded_data.is_null() || decoded_data_bytes <= 0i32 as libc::c_ulong {
            return 0i32;
        }
    } else {
        let mut r: libc::c_int = 0;
        let mut current_index: size_t = 0i32 as size_t;
        r = mailmime_part_parse(
            (*mime_data).dt_data.dt_text.dt_data,
            (*mime_data).dt_data.dt_text.dt_length,
            &mut current_index,
            mime_transfer_encoding,
            &mut transfer_decoding_buffer,
            &mut decoded_data_bytes,
        );
        if r != MAILIMF_NO_ERROR as libc::c_int
            || transfer_decoding_buffer.is_null()
            || decoded_data_bytes <= 0i32 as libc::c_ulong
        {
            return 0i32;
        }
        decoded_data = transfer_decoding_buffer
    }
    *ret_decoded_data = decoded_data;
    *ret_decoded_data_bytes = decoded_data_bytes;
    *ret_to_mmap_string_unref = transfer_decoding_buffer;
    return 1i32;
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_is_mailinglist_message(
    mut mimeparser: *mut dc_mimeparser_t,
) -> libc::c_int {
    if mimeparser.is_null() {
        return 0i32;
    }
    if !dc_mimeparser_lookup_field(
        mimeparser,
        b"List-Id\x00" as *const u8 as *const libc::c_char,
    )
    .is_null()
    {
        return 1i32;
    }
    let mut precedence: *mut mailimf_optional_field = dc_mimeparser_lookup_optional_field(
        mimeparser,
        b"Precedence\x00" as *const u8 as *const libc::c_char,
    );
    if !precedence.is_null() {
        if strcasecmp(
            (*precedence).fld_value,
            b"list\x00" as *const u8 as *const libc::c_char,
        ) == 0i32
            || strcasecmp(
                (*precedence).fld_value,
                b"bulk\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
        {
            return 1i32;
        }
    }
    return 0i32;
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_sender_equals_recipient(
    mut mimeparser: *mut dc_mimeparser_t,
) -> libc::c_int {
    let mut sender_equals_recipient: libc::c_int = 0i32;
    let mut fld: *const mailimf_field = 0 as *const mailimf_field;
    let mut fld_from: *const mailimf_from = 0 as *const mailimf_from;
    let mut mb: *mut mailimf_mailbox = 0 as *mut mailimf_mailbox;
    let mut from_addr_norm: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut recipients: *mut dc_hash_t = 0 as *mut dc_hash_t;
    if !(mimeparser.is_null() || (*mimeparser).header_root.is_null()) {
        /* get From: and check there is exactly one sender */
        fld = mailimf_find_field((*mimeparser).header_root, MAILIMF_FIELD_FROM as libc::c_int);
        if !(fld.is_null()
            || {
                fld_from = (*fld).fld_data.fld_from;
                fld_from.is_null()
            }
            || (*fld_from).frm_mb_list.is_null()
            || (*(*fld_from).frm_mb_list).mb_list.is_null()
            || (*(*(*fld_from).frm_mb_list).mb_list).count != 1i32)
        {
            mb = (if !(*(*(*fld_from).frm_mb_list).mb_list).first.is_null() {
                (*(*(*(*fld_from).frm_mb_list).mb_list).first).data
            } else {
                0 as *mut libc::c_void
            }) as *mut mailimf_mailbox;
            if !mb.is_null() {
                from_addr_norm = dc_addr_normalize((*mb).mb_addr_spec);
                recipients = mailimf_get_recipients((*mimeparser).header_root);
                if !((*recipients).count != 1i32) {
                    if !dc_hash_find(
                        recipients,
                        from_addr_norm as *const libc::c_void,
                        strlen(from_addr_norm) as libc::c_int,
                    )
                    .is_null()
                    {
                        sender_equals_recipient = 1i32
                    }
                }
            }
        }
    }
    dc_hash_clear(recipients);
    free(recipients as *mut libc::c_void);
    free(from_addr_norm as *mut libc::c_void);
    return sender_equals_recipient;
}
#[no_mangle]
pub unsafe extern "C" fn mailimf_get_recipients(
    mut imffields: *mut mailimf_fields,
) -> *mut dc_hash_t {
    /* the returned value must be dc_hash_clear()'d and free()'d. returned addresses are normalized. */
    let mut recipients: *mut dc_hash_t =
        malloc(::std::mem::size_of::<dc_hash_t>() as libc::c_ulong) as *mut dc_hash_t;
    dc_hash_init(recipients, 3i32, 1i32);
    let mut cur1: *mut clistiter = 0 as *mut clistiter;
    cur1 = (*(*imffields).fld_list).first;
    while !cur1.is_null() {
        let mut fld: *mut mailimf_field = (if !cur1.is_null() {
            (*cur1).data
        } else {
            0 as *mut libc::c_void
        }) as *mut mailimf_field;
        let mut fld_to: *mut mailimf_to = 0 as *mut mailimf_to;
        let mut fld_cc: *mut mailimf_cc = 0 as *mut mailimf_cc;
        let mut addr_list: *mut mailimf_address_list = 0 as *mut mailimf_address_list;
        match (*fld).fld_type {
            13 => {
                fld_to = (*fld).fld_data.fld_to;
                if !fld_to.is_null() {
                    addr_list = (*fld_to).to_addr_list
                }
            }
            14 => {
                fld_cc = (*fld).fld_data.fld_cc;
                if !fld_cc.is_null() {
                    addr_list = (*fld_cc).cc_addr_list
                }
            }
            _ => {}
        }
        if !addr_list.is_null() {
            let mut cur2: *mut clistiter = 0 as *mut clistiter;
            cur2 = (*(*addr_list).ad_list).first;
            while !cur2.is_null() {
                let mut adr: *mut mailimf_address = (if !cur2.is_null() {
                    (*cur2).data
                } else {
                    0 as *mut libc::c_void
                }) as *mut mailimf_address;
                if !adr.is_null() {
                    if (*adr).ad_type == MAILIMF_ADDRESS_MAILBOX as libc::c_int {
                        mailimf_get_recipients__add_addr(recipients, (*adr).ad_data.ad_mailbox);
                    } else if (*adr).ad_type == MAILIMF_ADDRESS_GROUP as libc::c_int {
                        let mut group: *mut mailimf_group = (*adr).ad_data.ad_group;
                        if !group.is_null() && !(*group).grp_mb_list.is_null() {
                            let mut cur3: *mut clistiter = 0 as *mut clistiter;
                            cur3 = (*(*(*group).grp_mb_list).mb_list).first;
                            while !cur3.is_null() {
                                mailimf_get_recipients__add_addr(
                                    recipients,
                                    (if !cur3.is_null() {
                                        (*cur3).data
                                    } else {
                                        0 as *mut libc::c_void
                                    }) as *mut mailimf_mailbox,
                                );
                                cur3 = if !cur3.is_null() {
                                    (*cur3).next
                                } else {
                                    0 as *mut clistcell_s
                                }
                            }
                        }
                    }
                }
                cur2 = if !cur2.is_null() {
                    (*cur2).next
                } else {
                    0 as *mut clistcell_s
                }
            }
        }
        cur1 = if !cur1.is_null() {
            (*cur1).next
        } else {
            0 as *mut clistcell_s
        }
    }
    return recipients;
}
/* ******************************************************************************
 * debug output
 ******************************************************************************/
/* DEBUG_MIME_OUTPUT */
/* ******************************************************************************
 * low-level-tools for getting a list of all recipients
 ******************************************************************************/
unsafe extern "C" fn mailimf_get_recipients__add_addr(
    mut recipients: *mut dc_hash_t,
    mut mb: *mut mailimf_mailbox,
) {
    if !mb.is_null() {
        let mut addr_norm: *mut libc::c_char = dc_addr_normalize((*mb).mb_addr_spec);
        dc_hash_insert(
            recipients,
            addr_norm as *const libc::c_void,
            strlen(addr_norm) as libc::c_int,
            1i32 as *mut libc::c_void,
        );
        free(addr_norm as *mut libc::c_void);
    };
}
/*the result is a pointer to mime, must not be freed*/
#[no_mangle]
pub unsafe extern "C" fn mailimf_find_field(
    mut header: *mut mailimf_fields,
    mut wanted_fld_type: libc::c_int,
) -> *mut mailimf_field {
    if header.is_null() || (*header).fld_list.is_null() {
        return 0 as *mut mailimf_field;
    }
    let mut cur1: *mut clistiter = (*(*header).fld_list).first;
    while !cur1.is_null() {
        let mut field: *mut mailimf_field = (if !cur1.is_null() {
            (*cur1).data
        } else {
            0 as *mut libc::c_void
        }) as *mut mailimf_field;
        if !field.is_null() {
            if (*field).fld_type == wanted_fld_type {
                return field;
            }
        }
        cur1 = if !cur1.is_null() {
            (*cur1).next
        } else {
            0 as *mut clistcell_s
        }
    }
    return 0 as *mut mailimf_field;
}
#[no_mangle]
pub unsafe extern "C" fn dc_mimeparser_repl_msg_by_error(
    mut mimeparser: *mut dc_mimeparser_t,
    mut error_msg: *const libc::c_char,
) {
    let mut part: *mut dc_mimepart_t = 0 as *mut dc_mimepart_t;
    let mut i: libc::c_int = 0i32;
    if mimeparser.is_null()
        || (*mimeparser).parts.is_null()
        || carray_count((*mimeparser).parts) <= 0i32 as libc::c_uint
    {
        return;
    }
    part = carray_get((*mimeparser).parts, 0i32 as libc::c_uint) as *mut dc_mimepart_t;
    (*part).type_0 = 10i32;
    free((*part).msg as *mut libc::c_void);
    (*part).msg = dc_mprintf(b"[%s]\x00" as *const u8 as *const libc::c_char, error_msg);
    i = 1i32;
    while (i as libc::c_uint) < carray_count((*mimeparser).parts) {
        part = carray_get((*mimeparser).parts, i as libc::c_uint) as *mut dc_mimepart_t;
        if !part.is_null() {
            dc_mimepart_unref(part);
        }
        i += 1
    }
    carray_set_size((*mimeparser).parts, 1i32 as libc::c_uint);
}
/*the result is a pointer to mime, must not be freed*/
#[no_mangle]
pub unsafe extern "C" fn mailmime_find_mailimf_fields(
    mut mime: *mut mailmime,
) -> *mut mailimf_fields {
    if mime.is_null() {
        return 0 as *mut mailimf_fields;
    }
    match (*mime).mm_type {
        2 => {
            let mut cur: *mut clistiter = (*(*mime).mm_data.mm_multipart.mm_mp_list).first;
            while !cur.is_null() {
                let mut header: *mut mailimf_fields = mailmime_find_mailimf_fields(
                    (if !cur.is_null() {
                        (*cur).data
                    } else {
                        0 as *mut libc::c_void
                    }) as *mut mailmime,
                );
                if !header.is_null() {
                    return header;
                }
                cur = if !cur.is_null() {
                    (*cur).next
                } else {
                    0 as *mut clistcell_s
                }
            }
        }
        3 => return (*mime).mm_data.mm_message.mm_fields,
        _ => {}
    }
    return 0 as *mut mailimf_fields;
}
#[no_mangle]
pub unsafe extern "C" fn mailimf_find_optional_field(
    mut header: *mut mailimf_fields,
    mut wanted_fld_name: *const libc::c_char,
) -> *mut mailimf_optional_field {
    if header.is_null() || (*header).fld_list.is_null() {
        return 0 as *mut mailimf_optional_field;
    }
    let mut cur1: *mut clistiter = (*(*header).fld_list).first;
    while !cur1.is_null() {
        let mut field: *mut mailimf_field = (if !cur1.is_null() {
            (*cur1).data
        } else {
            0 as *mut libc::c_void
        }) as *mut mailimf_field;
        if !field.is_null() && (*field).fld_type == MAILIMF_FIELD_OPTIONAL_FIELD as libc::c_int {
            let mut optional_field: *mut mailimf_optional_field =
                (*field).fld_data.fld_optional_field;
            if !optional_field.is_null()
                && !(*optional_field).fld_name.is_null()
                && !(*optional_field).fld_value.is_null()
                && strcasecmp((*optional_field).fld_name, wanted_fld_name) == 0i32
            {
                return optional_field;
            }
        }
        cur1 = if !cur1.is_null() {
            (*cur1).next
        } else {
            0 as *mut clistcell_s
        }
    }
    return 0 as *mut mailimf_optional_field;
}