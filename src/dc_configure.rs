use c2rust_bitfields::BitfieldStruct;
use libc;

use crate::dc_array::*;
use crate::dc_context::dc_context_t;
use crate::dc_e2ee::*;
use crate::dc_imap::*;
use crate::dc_job::*;
use crate::dc_jobthread::dc_jobthread_t;
use crate::dc_log::*;
use crate::dc_loginparam::*;
use crate::dc_lot::dc_lot_t;
use crate::dc_oauth2::*;
use crate::dc_saxparser::*;
use crate::dc_smtp::*;
use crate::dc_sqlite3::*;
use crate::dc_stock::*;
use crate::dc_strencode::*;
use crate::dc_tools::*;
use crate::types::*;
use crate::x::*;

/* ******************************************************************************
 * Configure folders
 ******************************************************************************/
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dc_imapfolder_t {
    pub name_to_select: *mut libc::c_char,
    pub name_utf8: *mut libc::c_char,
    pub meaning: libc::c_int,
}
/* ******************************************************************************
 * Thunderbird's Autoconfigure
 ******************************************************************************/
/* documentation: https://developer.mozilla.org/en-US/docs/Mozilla/Thunderbird/Autoconfiguration */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct moz_autoconfigure_t {
    pub in_0: *const dc_loginparam_t,
    pub in_emaildomain: *mut libc::c_char,
    pub in_emaillocalpart: *mut libc::c_char,
    pub out: *mut dc_loginparam_t,
    pub out_imap_set: libc::c_int,
    pub out_smtp_set: libc::c_int,
    pub tag_server: libc::c_int,
    pub tag_config: libc::c_int,
}

/* ******************************************************************************
 * Outlook's Autodiscover
 ******************************************************************************/
#[derive(Copy, Clone)]
#[repr(C)]
pub struct outlk_autodiscover_t {
    pub in_0: *const dc_loginparam_t,
    pub out: *mut dc_loginparam_t,
    pub out_imap_set: libc::c_int,
    pub out_smtp_set: libc::c_int,
    pub tag_config: libc::c_int,
    pub config: [*mut libc::c_char; 6],
    pub redirect: *mut libc::c_char,
}
// connect
#[no_mangle]
pub unsafe extern "C" fn dc_configure(mut context: *mut dc_context_t) {
    if 0 != dc_has_ongoing(context) {
        dc_log_warning(
            context,
            0i32,
            b"There is already another ongoing process running.\x00" as *const u8
                as *const libc::c_char,
        );
        return;
    }
    dc_job_kill_action(context, 900i32);
    dc_job_add(context, 900i32, 0i32, 0 as *const libc::c_char, 0i32);
}
#[no_mangle]
pub unsafe extern "C" fn dc_has_ongoing(mut context: *mut dc_context_t) -> libc::c_int {
    if context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint {
        return 0i32;
    }
    return if 0 != (*context).ongoing_running || (*context).shall_stop_ongoing == 0i32 {
        1i32
    } else {
        0i32
    };
}
#[no_mangle]
pub unsafe extern "C" fn dc_is_configured(mut context: *const dc_context_t) -> libc::c_int {
    if context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint {
        return 0i32;
    }
    return if 0
        != dc_sqlite3_get_config_int(
            (*context).sql,
            b"configured\x00" as *const u8 as *const libc::c_char,
            0i32,
        ) {
        1i32
    } else {
        0i32
    };
}
#[no_mangle]
pub unsafe extern "C" fn dc_stop_ongoing_process(mut context: *mut dc_context_t) {
    if context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint {
        return;
    }
    if 0 != (*context).ongoing_running && (*context).shall_stop_ongoing == 0i32 {
        dc_log_info(
            context,
            0i32,
            b"Signaling the ongoing process to stop ASAP.\x00" as *const u8 as *const libc::c_char,
        );
        (*context).shall_stop_ongoing = 1i32
    } else {
        dc_log_info(
            context,
            0i32,
            b"No ongoing process to stop.\x00" as *const u8 as *const libc::c_char,
        );
    };
}
// the other dc_job_do_DC_JOB_*() functions are declared static in the c-file
#[no_mangle]
pub unsafe extern "C" fn dc_job_do_DC_JOB_CONFIGURE_IMAP(
    mut context: *mut dc_context_t,
    mut job: *mut dc_job_t,
) {
    let mut flags: libc::c_int = 0;
    let mut current_block: u64;
    let mut success: libc::c_int = 0i32;
    let mut imap_connected_here: libc::c_int = 0i32;
    let mut smtp_connected_here: libc::c_int = 0i32;
    let mut ongoing_allocated_here: libc::c_int = 0i32;
    let mut mvbox_folder: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut param: *mut dc_loginparam_t = 0 as *mut dc_loginparam_t;
    /* just a pointer inside param, must not be freed! */
    let mut param_domain: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut param_addr_urlencoded: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut param_autoconfig: *mut dc_loginparam_t = 0 as *mut dc_loginparam_t;
    if !(context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint) {
        if !(0 == dc_alloc_ongoing(context)) {
            ongoing_allocated_here = 1i32;
            if 0 == dc_sqlite3_is_open((*context).sql) {
                dc_log_error(
                    context,
                    0i32,
                    b"Cannot configure, database not opened.\x00" as *const u8
                        as *const libc::c_char,
                );
            } else {
                dc_imap_disconnect((*context).inbox);
                dc_imap_disconnect((*context).sentbox_thread.imap);
                dc_imap_disconnect((*context).mvbox_thread.imap);
                dc_smtp_disconnect((*context).smtp);
                (*(*context).smtp).log_connect_errors = 1i32;
                (*(*context).inbox).log_connect_errors = 1i32;
                (*(*context).sentbox_thread.imap).log_connect_errors = 1i32;
                (*(*context).mvbox_thread.imap).log_connect_errors = 1i32;
                dc_log_info(
                    context,
                    0i32,
                    b"Configure ...\x00" as *const u8 as *const libc::c_char,
                );
                if !(0 != (*context).shall_stop_ongoing) {
                    (*context).cb.expect("non-null function pointer")(
                        context,
                        2041i32,
                        (if 0i32 < 1i32 {
                            1i32
                        } else if 0i32 > 999i32 {
                            999i32
                        } else {
                            0i32
                        }) as uintptr_t,
                        0i32 as uintptr_t,
                    );
                    param = dc_loginparam_new();
                    dc_loginparam_read(
                        param,
                        (*context).sql,
                        b"\x00" as *const u8 as *const libc::c_char,
                    );
                    if (*param).addr.is_null() {
                        dc_log_error(
                            context,
                            0i32,
                            b"Please enter the email address.\x00" as *const u8
                                as *const libc::c_char,
                        );
                    } else {
                        dc_trim((*param).addr);
                        if 0 != (*param).server_flags & 0x2i32 {
                            // the used oauth2 addr may differ, check this.
                            // if dc_get_oauth2_addr() is not available in the oauth2 implementation,
                            // just use the given one.
                            if 0 != (*context).shall_stop_ongoing {
                                current_block = 2927484062889439186;
                            } else {
                                (*context).cb.expect("non-null function pointer")(
                                    context,
                                    2041i32,
                                    (if 10i32 < 1i32 {
                                        1i32
                                    } else if 10i32 > 999i32 {
                                        999i32
                                    } else {
                                        10i32
                                    }) as uintptr_t,
                                    0i32 as uintptr_t,
                                );
                                let mut oauth2_addr: *mut libc::c_char =
                                    dc_get_oauth2_addr(context, (*param).addr, (*param).mail_pw);
                                if !oauth2_addr.is_null() {
                                    free((*param).addr as *mut libc::c_void);
                                    (*param).addr = oauth2_addr;
                                    dc_sqlite3_set_config(
                                        (*context).sql,
                                        b"addr\x00" as *const u8 as *const libc::c_char,
                                        (*param).addr,
                                    );
                                }
                                if 0 != (*context).shall_stop_ongoing {
                                    current_block = 2927484062889439186;
                                } else {
                                    (*context).cb.expect("non-null function pointer")(
                                        context,
                                        2041i32,
                                        (if 20i32 < 1i32 {
                                            1i32
                                        } else if 20i32 > 999i32 {
                                            999i32
                                        } else {
                                            20i32
                                        }) as uintptr_t,
                                        0i32 as uintptr_t,
                                    );
                                    current_block = 7746103178988627676;
                                }
                            }
                        } else {
                            current_block = 7746103178988627676;
                        }
                        match current_block {
                            2927484062889439186 => {}
                            _ => {
                                param_domain = strchr((*param).addr, '@' as i32);
                                if param_domain.is_null()
                                    || *param_domain.offset(0isize) as libc::c_int == 0i32
                                {
                                    dc_log_error(
                                        context,
                                        0i32,
                                        b"Bad email-address.\x00" as *const u8
                                            as *const libc::c_char,
                                    );
                                } else {
                                    param_domain = param_domain.offset(1isize);
                                    param_addr_urlencoded = dc_urlencode((*param).addr);
                                    if (*param).mail_pw.is_null() {
                                        (*param).mail_pw = dc_strdup(0 as *const libc::c_char)
                                    }
                                    if !(0 != (*context).shall_stop_ongoing) {
                                        (*context).cb.expect("non-null function pointer")(
                                            context,
                                            2041i32,
                                            (if 200i32 < 1i32 {
                                                1i32
                                            } else if 200i32 > 999i32 {
                                                999i32
                                            } else {
                                                200i32
                                            })
                                                as uintptr_t,
                                            0i32 as uintptr_t,
                                        );
                                        /* 2.  Autoconfig
                                         **************************************************************************/
                                        if (*param).mail_server.is_null()
                                            && (*param).mail_port as libc::c_int == 0i32
                                            && (*param).send_server.is_null()
                                            && (*param).send_port == 0i32
                                            && (*param).send_user.is_null()
                                            && (*param).server_flags & !0x2i32 == 0i32
                                        {
                                            /*&&param->mail_user   ==NULL -- the user can enter a loginname which is used by autoconfig then */
                                            /*&&param->send_pw     ==NULL -- the password cannot be auto-configured and is no criterion for autoconfig or not */
                                            /* flags but OAuth2 avoid autoconfig */
                                            let mut keep_flags: libc::c_int =
                                                (*param).server_flags & 0x2i32;
                                            /* A.  Search configurations from the domain used in the email-address, prefer encrypted */
                                            if param_autoconfig.is_null() {
                                                let mut url:
                                                        *mut libc::c_char =
                                                    dc_mprintf(b"https://autoconfig.%s/mail/config-v1.1.xml?emailaddress=%s\x00"
                                                                   as
                                                                   *const u8
                                                                   as
                                                                   *const libc::c_char,
                                                               param_domain,
                                                               param_addr_urlencoded);
                                                param_autoconfig =
                                                    moz_autoconfigure(context, url, param);
                                                free(url as *mut libc::c_void);
                                                if 0 != (*context).shall_stop_ongoing {
                                                    current_block = 2927484062889439186;
                                                } else {
                                                    (*context)
                                                        .cb
                                                        .expect("non-null function pointer")(
                                                        context,
                                                        2041i32,
                                                        (if 300i32 < 1i32 {
                                                            1i32
                                                        } else if 300i32 > 999i32 {
                                                            999i32
                                                        } else {
                                                            300i32
                                                        })
                                                            as uintptr_t,
                                                        0i32 as uintptr_t,
                                                    );
                                                    current_block = 13325891313334703151;
                                                }
                                            } else {
                                                current_block = 13325891313334703151;
                                            }
                                            match current_block {
                                                2927484062889439186 => {}
                                                _ => {
                                                    if param_autoconfig.is_null() {
                                                        // the doc does not mention `emailaddress=`, however, Thunderbird adds it, see https://releases.mozilla.org/pub/thunderbird/ ,  which makes some sense
                                                        let mut url_0:
                                                                *mut libc::c_char =
                                                            dc_mprintf(b"https://%s/.well-known/autoconfig/mail/config-v1.1.xml?emailaddress=%s\x00"
                                                                           as
                                                                           *const u8
                                                                           as
                                                                           *const libc::c_char,
                                                                       param_domain,
                                                                       param_addr_urlencoded);
                                                        param_autoconfig = moz_autoconfigure(
                                                            context, url_0, param,
                                                        );
                                                        free(url_0 as *mut libc::c_void);
                                                        if 0 != (*context).shall_stop_ongoing {
                                                            current_block = 2927484062889439186;
                                                        } else {
                                                            (*context).cb.expect(
                                                                "non-null function pointer",
                                                            )(
                                                                context,
                                                                2041i32,
                                                                (if 310i32 < 1i32 {
                                                                    1i32
                                                                } else if 310i32 > 999i32 {
                                                                    999i32
                                                                } else {
                                                                    310i32
                                                                })
                                                                    as uintptr_t,
                                                                0i32 as uintptr_t,
                                                            );
                                                            current_block = 5597585068398118923;
                                                        }
                                                    } else {
                                                        current_block = 5597585068398118923;
                                                    }
                                                    match current_block {
                                                        2927484062889439186 => {}
                                                        _ => {
                                                            let mut i: libc::c_int = 0i32;
                                                            loop {
                                                                if !(i <= 1i32) {
                                                                    current_block =
                                                                        12961834331865314435;
                                                                    break;
                                                                }
                                                                if param_autoconfig.is_null() {
                                                                    /* Outlook uses always SSL but different domains */
                                                                    let mut url_1:
                                                                            *mut libc::c_char =
                                                                        dc_mprintf(b"https://%s%s/autodiscover/autodiscover.xml\x00"
                                                                                       as
                                                                                       *const u8
                                                                                       as
                                                                                       *const libc::c_char,
                                                                                   if i
                                                                                          ==
                                                                                          0i32
                                                                                      {
                                                                                       b"\x00"
                                                                                           as
                                                                                           *const u8
                                                                                           as
                                                                                           *const libc::c_char
                                                                                   } else {
                                                                                       b"autodiscover.\x00"
                                                                                           as
                                                                                           *const u8
                                                                                           as
                                                                                           *const libc::c_char
                                                                                   },
                                                                                   param_domain);
                                                                    param_autoconfig =
                                                                        outlk_autodiscover(
                                                                            context, url_1, param,
                                                                        );
                                                                    free(
                                                                        url_1 as *mut libc::c_void,
                                                                    );
                                                                    if 0 != (*context)
                                                                        .shall_stop_ongoing
                                                                    {
                                                                        current_block =
                                                                            2927484062889439186;
                                                                        break;
                                                                    }
                                                                    (*context).cb.expect(
                                                                        "non-null function pointer",
                                                                    )(
                                                                        context,
                                                                        2041i32,
                                                                        (if 320i32 + i * 10i32
                                                                            < 1i32
                                                                        {
                                                                            1i32
                                                                        } else if 320i32 + i * 10i32
                                                                            > 999i32
                                                                        {
                                                                            999i32
                                                                        } else {
                                                                            320i32 + i * 10i32
                                                                        })
                                                                            as uintptr_t,
                                                                        0i32 as uintptr_t,
                                                                    );
                                                                }
                                                                i += 1
                                                            }
                                                            match current_block {
                                                                2927484062889439186 => {}
                                                                _ => {
                                                                    if param_autoconfig.is_null() {
                                                                        let mut url_2:
                                                                                *mut libc::c_char =
                                                                            dc_mprintf(b"http://autoconfig.%s/mail/config-v1.1.xml?emailaddress=%s\x00"
                                                                                           as
                                                                                           *const u8
                                                                                           as
                                                                                           *const libc::c_char,
                                                                                       param_domain,
                                                                                       param_addr_urlencoded);
                                                                        param_autoconfig =
                                                                            moz_autoconfigure(
                                                                                context, url_2,
                                                                                param,
                                                                            );
                                                                        free(url_2
                                                                                 as
                                                                                 *mut libc::c_void);
                                                                        if 0 != (*context)
                                                                            .shall_stop_ongoing
                                                                        {
                                                                            current_block =
                                                                                2927484062889439186;
                                                                        } else {
                                                                            (*context).cb.expect("non-null function pointer")(context,
                                                                                                                              2041i32,
                                                                                                                              (if 340i32
                                                                                                                                      <
                                                                                                                                      1i32
                                                                                                                                  {
                                                                                                                                   1i32
                                                                                                                               } else if 340i32
                                                                                                                                             >
                                                                                                                                             999i32
                                                                                                                                {
                                                                                                                                   999i32
                                                                                                                               } else {
                                                                                                                                   340i32
                                                                                                                               })
                                                                                                                                  as
                                                                                                                                  uintptr_t,
                                                                                                                              0i32
                                                                                                                                  as
                                                                                                                                  uintptr_t);
                                                                            current_block
                                                                                =
                                                                                10778260831612459202;
                                                                        }
                                                                    } else {
                                                                        current_block =
                                                                            10778260831612459202;
                                                                    }
                                                                    match current_block {
                                                                        2927484062889439186 => {}
                                                                        _ => {
                                                                            if param_autoconfig
                                                                                .is_null()
                                                                            {
                                                                                // do not transfer the email-address unencrypted
                                                                                let mut url_3:
                                                                                        *mut libc::c_char =
                                                                                    dc_mprintf(b"http://%s/.well-known/autoconfig/mail/config-v1.1.xml\x00"
                                                                                                   as
                                                                                                   *const u8
                                                                                                   as
                                                                                                   *const libc::c_char,
                                                                                               param_domain);
                                                                                param_autoconfig
                                                                                    =
                                                                                    moz_autoconfigure(context,
                                                                                                      url_3,
                                                                                                      param);
                                                                                free(url_3
                                                                                         as
                                                                                         *mut libc::c_void);
                                                                                if 0
                                                                                       !=
                                                                                       (*context).shall_stop_ongoing
                                                                                   {
                                                                                    current_block
                                                                                        =
                                                                                        2927484062889439186;
                                                                                } else {
                                                                                    (*context).cb.expect("non-null function pointer")(context,
                                                                                                                                      2041i32,
                                                                                                                                      (if 350i32
                                                                                                                                              <
                                                                                                                                              1i32
                                                                                                                                          {
                                                                                                                                           1i32
                                                                                                                                       } else if 350i32
                                                                                                                                                     >
                                                                                                                                                     999i32
                                                                                                                                        {
                                                                                                                                           999i32
                                                                                                                                       } else {
                                                                                                                                           350i32
                                                                                                                                       })
                                                                                                                                          as
                                                                                                                                          uintptr_t,
                                                                                                                                      0i32
                                                                                                                                          as
                                                                                                                                          uintptr_t);
                                                                                    current_block
                                                                                        =
                                                                                        5207889489643863322;
                                                                                }
                                                                            } else {
                                                                                current_block
                                                                                    =
                                                                                    5207889489643863322;
                                                                            }
                                                                            match current_block
                                                                                {
                                                                                2927484062889439186
                                                                                =>
                                                                                {
                                                                                }
                                                                                _
                                                                                =>
                                                                                {
                                                                                    /* B.  If we have no configuration yet, search configuration in Thunderbird's centeral database */
                                                                                    if param_autoconfig.is_null()
                                                                                       {
                                                                                        /* always SSL for Thunderbird's database */
                                                                                        let mut url_4:
                                                                                                *mut libc::c_char =
                                                                                            dc_mprintf(b"https://autoconfig.thunderbird.net/v1.1/%s\x00"
                                                                                                           as
                                                                                                           *const u8
                                                                                                           as
                                                                                                           *const libc::c_char,
                                                                                                       param_domain);
                                                                                        param_autoconfig
                                                                                            =
                                                                                            moz_autoconfigure(context,
                                                                                                              url_4,
                                                                                                              param);
                                                                                        free(url_4
                                                                                                 as
                                                                                                 *mut libc::c_void);
                                                                                        if 0
                                                                                               !=
                                                                                               (*context).shall_stop_ongoing
                                                                                           {
                                                                                            current_block
                                                                                                =
                                                                                                2927484062889439186;
                                                                                        } else {
                                                                                            (*context).cb.expect("non-null function pointer")(context,
                                                                                                                                              2041i32,
                                                                                                                                              (if 500i32
                                                                                                                                                      <
                                                                                                                                                      1i32
                                                                                                                                                  {
                                                                                                                                                   1i32
                                                                                                                                               } else if 500i32
                                                                                                                                                             >
                                                                                                                                                             999i32
                                                                                                                                                {
                                                                                                                                                   999i32
                                                                                                                                               } else {
                                                                                                                                                   500i32
                                                                                                                                               })
                                                                                                                                                  as
                                                                                                                                                  uintptr_t,
                                                                                                                                              0i32
                                                                                                                                                  as
                                                                                                                                                  uintptr_t);
                                                                                            current_block
                                                                                                =
                                                                                                2798392256336243897;
                                                                                        }
                                                                                    } else {
                                                                                        current_block
                                                                                            =
                                                                                            2798392256336243897;
                                                                                    }
                                                                                    match current_block
                                                                                        {
                                                                                        2927484062889439186
                                                                                        =>
                                                                                        {
                                                                                        }
                                                                                        _
                                                                                        =>
                                                                                        {
                                                                                            if !param_autoconfig.is_null()
                                                                                               {
                                                                                                let mut r:
                                                                                                        *mut libc::c_char =
                                                                                                    dc_loginparam_get_readable(param_autoconfig);
                                                                                                dc_log_info(context,
                                                                                                            0i32,
                                                                                                            b"Got autoconfig: %s\x00"
                                                                                                                as
                                                                                                                *const u8
                                                                                                                as
                                                                                                                *const libc::c_char,
                                                                                                            r);
                                                                                                free(r
                                                                                                         as
                                                                                                         *mut libc::c_void);
                                                                                                if !(*param_autoconfig).mail_user.is_null()
                                                                                                   {
                                                                                                    free((*param).mail_user
                                                                                                             as
                                                                                                             *mut libc::c_void);
                                                                                                    (*param).mail_user
                                                                                                        =
                                                                                                        dc_strdup_keep_null((*param_autoconfig).mail_user)
                                                                                                }
                                                                                                (*param).mail_server
                                                                                                    =
                                                                                                    dc_strdup_keep_null((*param_autoconfig).mail_server);
                                                                                                (*param).mail_port
                                                                                                    =
                                                                                                    (*param_autoconfig).mail_port;
                                                                                                (*param).send_server
                                                                                                    =
                                                                                                    dc_strdup_keep_null((*param_autoconfig).send_server);
                                                                                                (*param).send_port
                                                                                                    =
                                                                                                    (*param_autoconfig).send_port;
                                                                                                (*param).send_user
                                                                                                    =
                                                                                                    dc_strdup_keep_null((*param_autoconfig).send_user);
                                                                                                (*param).server_flags
                                                                                                    =
                                                                                                    (*param_autoconfig).server_flags
                                                                                            }
                                                                                            (*param).server_flags
                                                                                                |=
                                                                                                keep_flags;
                                                                                            current_block
                                                                                                =
                                                                                                3024367268842933116;
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            current_block = 3024367268842933116;
                                        }
                                        match current_block {
                                            2927484062889439186 => {}
                                            _ => {
                                                if (*param).mail_server.is_null() {
                                                    (*param).mail_server = dc_mprintf(
                                                        b"imap.%s\x00" as *const u8
                                                            as *const libc::c_char,
                                                        param_domain,
                                                    )
                                                }
                                                if (*param).mail_port as libc::c_int == 0i32 {
                                                    (*param).mail_port = (if 0
                                                        != (*param).server_flags
                                                            & (0x100i32 | 0x400i32)
                                                    {
                                                        143i32
                                                    } else {
                                                        993i32
                                                    })
                                                        as uint16_t
                                                }
                                                if (*param).mail_user.is_null() {
                                                    (*param).mail_user = dc_strdup((*param).addr)
                                                }
                                                if (*param).send_server.is_null()
                                                    && !(*param).mail_server.is_null()
                                                {
                                                    (*param).send_server =
                                                        dc_strdup((*param).mail_server);
                                                    if strncmp(
                                                        (*param).send_server,
                                                        b"imap.\x00" as *const u8
                                                            as *const libc::c_char,
                                                        5i32 as libc::c_ulong,
                                                    ) == 0i32
                                                    {
                                                        memcpy(
                                                            (*param).send_server
                                                                as *mut libc::c_void,
                                                            b"smtp\x00" as *const u8
                                                                as *const libc::c_char
                                                                as *const libc::c_void,
                                                            4i32 as libc::c_ulong,
                                                        );
                                                    }
                                                }
                                                if (*param).send_port == 0i32 {
                                                    (*param).send_port =
                                                        if 0 != (*param).server_flags & 0x10000i32 {
                                                            587i32
                                                        } else if 0
                                                            != (*param).server_flags & 0x40000i32
                                                        {
                                                            25i32
                                                        } else {
                                                            465i32
                                                        }
                                                }
                                                if (*param).send_user.is_null()
                                                    && !(*param).mail_user.is_null()
                                                {
                                                    (*param).send_user =
                                                        dc_strdup((*param).mail_user)
                                                }
                                                if (*param).send_pw.is_null()
                                                    && !(*param).mail_pw.is_null()
                                                {
                                                    (*param).send_pw = dc_strdup((*param).mail_pw)
                                                }
                                                if 0 == dc_exactly_one_bit_set(
                                                    (*param).server_flags & (0x2i32 | 0x4i32),
                                                ) {
                                                    (*param).server_flags &= !(0x2i32 | 0x4i32);
                                                    (*param).server_flags |= 0x4i32
                                                }
                                                if 0 == dc_exactly_one_bit_set(
                                                    (*param).server_flags
                                                        & (0x100i32 | 0x200i32 | 0x400i32),
                                                ) {
                                                    (*param).server_flags &=
                                                        !(0x100i32 | 0x200i32 | 0x400i32);
                                                    (*param).server_flags |=
                                                        if (*param).send_port == 143i32 {
                                                            0x100i32
                                                        } else {
                                                            0x200i32
                                                        }
                                                }
                                                if 0 == dc_exactly_one_bit_set(
                                                    (*param).server_flags
                                                        & (0x10000i32 | 0x20000i32 | 0x40000i32),
                                                ) {
                                                    (*param).server_flags &=
                                                        !(0x10000i32 | 0x20000i32 | 0x40000i32);
                                                    (*param).server_flags |=
                                                        if (*param).send_port == 587i32 {
                                                            0x10000i32
                                                        } else if (*param).send_port == 25i32 {
                                                            0x40000i32
                                                        } else {
                                                            0x20000i32
                                                        }
                                                }
                                                /* do we have a complete configuration? */
                                                if (*param).addr.is_null()
                                                    || (*param).mail_server.is_null()
                                                    || (*param).mail_port as libc::c_int == 0i32
                                                    || (*param).mail_user.is_null()
                                                    || (*param).mail_pw.is_null()
                                                    || (*param).send_server.is_null()
                                                    || (*param).send_port == 0i32
                                                    || (*param).send_user.is_null()
                                                    || (*param).send_pw.is_null()
                                                    || (*param).server_flags == 0i32
                                                {
                                                    dc_log_error(
                                                        context,
                                                        0i32,
                                                        b"Account settings incomplete.\x00"
                                                            as *const u8
                                                            as *const libc::c_char,
                                                    );
                                                } else if !(0 != (*context).shall_stop_ongoing) {
                                                    (*context)
                                                        .cb
                                                        .expect("non-null function pointer")(
                                                        context,
                                                        2041i32,
                                                        (if 600i32 < 1i32 {
                                                            1i32
                                                        } else if 600i32 > 999i32 {
                                                            999i32
                                                        } else {
                                                            600i32
                                                        })
                                                            as uintptr_t,
                                                        0i32 as uintptr_t,
                                                    );
                                                    /* try to connect to IMAP - if we did not got an autoconfig,
                                                    do some further tries with different settings and username variations */
                                                    let mut username_variation: libc::c_int = 0i32;
                                                    loop {
                                                        if !(username_variation <= 1i32) {
                                                            current_block = 14187386403465544025;
                                                            break;
                                                        }
                                                        let mut r_0: *mut libc::c_char =
                                                            dc_loginparam_get_readable(param);
                                                        dc_log_info(
                                                            context,
                                                            0i32,
                                                            b"Trying: %s\x00" as *const u8
                                                                as *const libc::c_char,
                                                            r_0,
                                                        );
                                                        free(r_0 as *mut libc::c_void);
                                                        if 0 != dc_imap_connect(
                                                            (*context).inbox,
                                                            param,
                                                        ) {
                                                            current_block = 14187386403465544025;
                                                            break;
                                                        }
                                                        if !param_autoconfig.is_null() {
                                                            current_block = 2927484062889439186;
                                                            break;
                                                        }
                                                        // probe STARTTLS/993
                                                        if 0 != (*context).shall_stop_ongoing {
                                                            current_block = 2927484062889439186;
                                                            break;
                                                        }
                                                        (*context)
                                                            .cb
                                                            .expect("non-null function pointer")(
                                                            context,
                                                            2041i32,
                                                            (if 650i32 + username_variation * 30i32
                                                                < 1i32
                                                            {
                                                                1i32
                                                            } else if 650i32
                                                                + username_variation * 30i32
                                                                > 999i32
                                                            {
                                                                999i32
                                                            } else {
                                                                650i32 + username_variation * 30i32
                                                            })
                                                                as uintptr_t,
                                                            0i32 as uintptr_t,
                                                        );
                                                        (*param).server_flags &=
                                                            !(0x100i32 | 0x200i32 | 0x400i32);
                                                        (*param).server_flags |= 0x100i32;
                                                        let mut r_1: *mut libc::c_char =
                                                            dc_loginparam_get_readable(param);
                                                        dc_log_info(
                                                            context,
                                                            0i32,
                                                            b"Trying: %s\x00" as *const u8
                                                                as *const libc::c_char,
                                                            r_1,
                                                        );
                                                        free(r_1 as *mut libc::c_void);
                                                        if 0 != dc_imap_connect(
                                                            (*context).inbox,
                                                            param,
                                                        ) {
                                                            current_block = 14187386403465544025;
                                                            break;
                                                        }
                                                        // probe STARTTLS/143
                                                        if 0 != (*context).shall_stop_ongoing {
                                                            current_block = 2927484062889439186;
                                                            break;
                                                        }
                                                        (*context)
                                                            .cb
                                                            .expect("non-null function pointer")(
                                                            context,
                                                            2041i32,
                                                            (if 660i32 + username_variation * 30i32
                                                                < 1i32
                                                            {
                                                                1i32
                                                            } else if 660i32
                                                                + username_variation * 30i32
                                                                > 999i32
                                                            {
                                                                999i32
                                                            } else {
                                                                660i32 + username_variation * 30i32
                                                            })
                                                                as uintptr_t,
                                                            0i32 as uintptr_t,
                                                        );
                                                        (*param).mail_port = 143i32 as uint16_t;
                                                        let mut r_2: *mut libc::c_char =
                                                            dc_loginparam_get_readable(param);
                                                        dc_log_info(
                                                            context,
                                                            0i32,
                                                            b"Trying: %s\x00" as *const u8
                                                                as *const libc::c_char,
                                                            r_2,
                                                        );
                                                        free(r_2 as *mut libc::c_void);
                                                        if 0 != dc_imap_connect(
                                                            (*context).inbox,
                                                            param,
                                                        ) {
                                                            current_block = 14187386403465544025;
                                                            break;
                                                        }
                                                        if 0 != username_variation {
                                                            current_block = 2927484062889439186;
                                                            break;
                                                        }
                                                        // next probe round with only the localpart of the email-address as the loginname
                                                        if 0 != (*context).shall_stop_ongoing {
                                                            current_block = 2927484062889439186;
                                                            break;
                                                        }
                                                        (*context)
                                                            .cb
                                                            .expect("non-null function pointer")(
                                                            context,
                                                            2041i32,
                                                            (if 670i32 + username_variation * 30i32
                                                                < 1i32
                                                            {
                                                                1i32
                                                            } else if 670i32
                                                                + username_variation * 30i32
                                                                > 999i32
                                                            {
                                                                999i32
                                                            } else {
                                                                670i32 + username_variation * 30i32
                                                            })
                                                                as uintptr_t,
                                                            0i32 as uintptr_t,
                                                        );
                                                        (*param).server_flags &=
                                                            !(0x100i32 | 0x200i32 | 0x400i32);
                                                        (*param).server_flags |= 0x200i32;
                                                        (*param).mail_port = 993i32 as uint16_t;
                                                        let mut at: *mut libc::c_char =
                                                            strchr((*param).mail_user, '@' as i32);
                                                        if !at.is_null() {
                                                            *at = 0i32 as libc::c_char
                                                        }
                                                        at = strchr((*param).send_user, '@' as i32);
                                                        if !at.is_null() {
                                                            *at = 0i32 as libc::c_char
                                                        }
                                                        username_variation += 1
                                                    }
                                                    match current_block {
                                                        2927484062889439186 => {}
                                                        _ => {
                                                            imap_connected_here = 1i32;
                                                            if !(0 != (*context).shall_stop_ongoing)
                                                            {
                                                                (*context).cb.expect(
                                                                    "non-null function pointer",
                                                                )(
                                                                    context,
                                                                    2041i32,
                                                                    (if 800i32 < 1i32 {
                                                                        1i32
                                                                    } else if 800i32 > 999i32 {
                                                                        999i32
                                                                    } else {
                                                                        800i32
                                                                    })
                                                                        as uintptr_t,
                                                                    0i32 as uintptr_t,
                                                                );
                                                                /* try to connect to SMTP - if we did not got an autoconfig, the first try was SSL-465 and we do a second try with STARTTLS-587 */
                                                                if 0 == dc_smtp_connect(
                                                                    (*context).smtp,
                                                                    param,
                                                                ) {
                                                                    if !param_autoconfig.is_null() {
                                                                        current_block =
                                                                            2927484062889439186;
                                                                    } else if 0
                                                                        != (*context)
                                                                            .shall_stop_ongoing
                                                                    {
                                                                        current_block =
                                                                            2927484062889439186;
                                                                    } else {
                                                                        (*context).cb.expect("non-null function pointer")(context,
                                                                                                                          2041i32,
                                                                                                                          (if 850i32
                                                                                                                                  <
                                                                                                                                  1i32
                                                                                                                              {
                                                                                                                               1i32
                                                                                                                           } else if 850i32
                                                                                                                                         >
                                                                                                                                         999i32
                                                                                                                            {
                                                                                                                               999i32
                                                                                                                           } else {
                                                                                                                               850i32
                                                                                                                           })
                                                                                                                              as
                                                                                                                              uintptr_t,
                                                                                                                          0i32
                                                                                                                              as
                                                                                                                              uintptr_t);
                                                                        (*param).server_flags &=
                                                                            !(0x10000i32
                                                                                | 0x20000i32
                                                                                | 0x40000i32);
                                                                        (*param).server_flags |=
                                                                            0x10000i32;
                                                                        (*param).send_port = 587i32;
                                                                        let mut r_3:
                                                                                *mut libc::c_char =
                                                                            dc_loginparam_get_readable(param);
                                                                        dc_log_info(context,
                                                                                    0i32,
                                                                                    b"Trying: %s\x00"
                                                                                        as
                                                                                        *const u8
                                                                                        as
                                                                                        *const libc::c_char,
                                                                                    r_3);
                                                                        free(r_3
                                                                                 as
                                                                                 *mut libc::c_void);
                                                                        if 0 == dc_smtp_connect(
                                                                            (*context).smtp,
                                                                            param,
                                                                        ) {
                                                                            if 0 != (*context)
                                                                                .shall_stop_ongoing
                                                                            {
                                                                                current_block
                                                                                    =
                                                                                    2927484062889439186;
                                                                            } else {
                                                                                (*context).cb.expect("non-null function pointer")(context,
                                                                                                                                  2041i32,
                                                                                                                                  (if 860i32
                                                                                                                                          <
                                                                                                                                          1i32
                                                                                                                                      {
                                                                                                                                       1i32
                                                                                                                                   } else if 860i32
                                                                                                                                                 >
                                                                                                                                                 999i32
                                                                                                                                    {
                                                                                                                                       999i32
                                                                                                                                   } else {
                                                                                                                                       860i32
                                                                                                                                   })
                                                                                                                                      as
                                                                                                                                      uintptr_t,
                                                                                                                                  0i32
                                                                                                                                      as
                                                                                                                                      uintptr_t);
                                                                                (*param).server_flags
                                                                                    &=
                                                                                    !(0x10000i32
                                                                                          |
                                                                                          0x20000i32
                                                                                          |
                                                                                          0x40000i32);
                                                                                (*param).server_flags
                                                                                    |=
                                                                                    0x10000i32;
                                                                                (*param)
                                                                                    .send_port =
                                                                                    25i32;
                                                                                let mut r_4:
                                                                                        *mut libc::c_char =
                                                                                    dc_loginparam_get_readable(param);
                                                                                dc_log_info(context,
                                                                                            0i32,
                                                                                            b"Trying: %s\x00"
                                                                                                as
                                                                                                *const u8
                                                                                                as
                                                                                                *const libc::c_char,
                                                                                            r_4);
                                                                                free(r_4
                                                                                         as
                                                                                         *mut libc::c_void);
                                                                                if 0
                                                                                       ==
                                                                                       dc_smtp_connect((*context).smtp,
                                                                                                       param)
                                                                                   {
                                                                                    current_block
                                                                                        =
                                                                                        2927484062889439186;
                                                                                } else {
                                                                                    current_block
                                                                                        =
                                                                                        5083741289379115417;
                                                                                }
                                                                            }
                                                                        } else {
                                                                            current_block =
                                                                                5083741289379115417;
                                                                        }
                                                                    }
                                                                } else {
                                                                    current_block =
                                                                        5083741289379115417;
                                                                }
                                                                match current_block {
                                                                    2927484062889439186 => {}
                                                                    _ => {
                                                                        smtp_connected_here = 1i32;
                                                                        if !(0
                                                                            != (*context)
                                                                                .shall_stop_ongoing)
                                                                        {
                                                                            (*context).cb.expect("non-null function pointer")(context,
                                                                                                                              2041i32,
                                                                                                                              (if 900i32
                                                                                                                                      <
                                                                                                                                      1i32
                                                                                                                                  {
                                                                                                                                   1i32
                                                                                                                               } else if 900i32
                                                                                                                                             >
                                                                                                                                             999i32
                                                                                                                                {
                                                                                                                                   999i32
                                                                                                                               } else {
                                                                                                                                   900i32
                                                                                                                               })
                                                                                                                                  as
                                                                                                                                  uintptr_t,
                                                                                                                              0i32
                                                                                                                                  as
                                                                                                                                  uintptr_t);
                                                                            flags
                                                                                =
                                                                                if 0
                                                                                       !=
                                                                                       dc_sqlite3_get_config_int((*context).sql,
                                                                                                                 b"mvbox_watch\x00"
                                                                                                                     as
                                                                                                                     *const u8
                                                                                                                     as
                                                                                                                     *const libc::c_char,
                                                                                                                 1i32)
                                                                                       ||
                                                                                       0
                                                                                           !=
                                                                                           dc_sqlite3_get_config_int((*context).sql,
                                                                                                                     b"mvbox_move\x00"
                                                                                                                         as
                                                                                                                         *const u8
                                                                                                                         as
                                                                                                                         *const libc::c_char,
                                                                                                                     1i32)
                                                                                   {
                                                                                    0x1i32
                                                                                } else {
                                                                                    0i32
                                                                                };
                                                                            dc_configure_folders(
                                                                                context,
                                                                                (*context).inbox,
                                                                                flags,
                                                                            );
                                                                            if !(0 != (*context)
                                                                                .shall_stop_ongoing)
                                                                            {
                                                                                (*context).cb.expect("non-null function pointer")(context,
                                                                                                                                  2041i32,
                                                                                                                                  (if 910i32
                                                                                                                                          <
                                                                                                                                          1i32
                                                                                                                                      {
                                                                                                                                       1i32
                                                                                                                                   } else if 910i32
                                                                                                                                                 >
                                                                                                                                                 999i32
                                                                                                                                    {
                                                                                                                                       999i32
                                                                                                                                   } else {
                                                                                                                                       910i32
                                                                                                                                   })
                                                                                                                                      as
                                                                                                                                      uintptr_t,
                                                                                                                                  0i32
                                                                                                                                      as
                                                                                                                                      uintptr_t);
                                                                                dc_loginparam_write(param,
                                                                                                    (*context).sql,
                                                                                                    b"configured_\x00"
                                                                                                        as
                                                                                                        *const u8
                                                                                                        as
                                                                                                        *const libc::c_char);
                                                                                dc_sqlite3_set_config_int((*context).sql,
                                                                                                          b"configured\x00"
                                                                                                              as
                                                                                                              *const u8
                                                                                                              as
                                                                                                              *const libc::c_char,
                                                                                                          1i32);
                                                                                if !(0
                                                                                         !=
                                                                                         (*context).shall_stop_ongoing)
                                                                                   {
                                                                                    (*context).cb.expect("non-null function pointer")(context,
                                                                                                                                      2041i32,
                                                                                                                                      (if 920i32
                                                                                                                                              <
                                                                                                                                              1i32
                                                                                                                                          {
                                                                                                                                           1i32
                                                                                                                                       } else if 920i32
                                                                                                                                                     >
                                                                                                                                                     999i32
                                                                                                                                        {
                                                                                                                                           999i32
                                                                                                                                       } else {
                                                                                                                                           920i32
                                                                                                                                       })
                                                                                                                                          as
                                                                                                                                          uintptr_t,
                                                                                                                                      0i32
                                                                                                                                          as
                                                                                                                                          uintptr_t);
                                                                                    dc_ensure_secret_key_exists(context);
                                                                                    success
                                                                                        =
                                                                                        1i32;
                                                                                    dc_log_info(context,
                                                                                                0i32,
                                                                                                b"Configure completed.\x00"
                                                                                                    as
                                                                                                    *const u8
                                                                                                    as
                                                                                                    *const libc::c_char);
                                                                                    if !(0
                                                                                             !=
                                                                                             (*context).shall_stop_ongoing)
                                                                                       {
                                                                                        (*context).cb.expect("non-null function pointer")(context,
                                                                                                                                          2041i32,
                                                                                                                                          (if 940i32
                                                                                                                                                  <
                                                                                                                                                  1i32
                                                                                                                                              {
                                                                                                                                               1i32
                                                                                                                                           } else if 940i32
                                                                                                                                                         >
                                                                                                                                                         999i32
                                                                                                                                            {
                                                                                                                                               999i32
                                                                                                                                           } else {
                                                                                                                                               940i32
                                                                                                                                           })
                                                                                                                                              as
                                                                                                                                              uintptr_t,
                                                                                                                                          0i32
                                                                                                                                              as
                                                                                                                                              uintptr_t);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if 0 != imap_connected_here {
        dc_imap_disconnect((*context).inbox);
    }
    if 0 != smtp_connected_here {
        dc_smtp_disconnect((*context).smtp);
    }
    dc_loginparam_unref(param);
    dc_loginparam_unref(param_autoconfig);
    free(param_addr_urlencoded as *mut libc::c_void);
    if 0 != ongoing_allocated_here {
        dc_free_ongoing(context);
    }
    free(mvbox_folder as *mut libc::c_void);
    (*context).cb.expect("non-null function pointer")(
        context,
        2041i32,
        (if 0 != success { 1000i32 } else { 0i32 }) as uintptr_t,
        0i32 as uintptr_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn dc_free_ongoing(mut context: *mut dc_context_t) {
    if context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint {
        return;
    }
    (*context).ongoing_running = 0i32;
    (*context).shall_stop_ongoing = 1i32;
}
#[no_mangle]
pub unsafe extern "C" fn dc_configure_folders(
    mut context: *mut dc_context_t,
    mut imap: *mut dc_imap_t,
    mut flags: libc::c_int,
) {
    let mut folder_list: *mut clist = 0 as *mut clist;
    let mut iter: *mut clistiter = 0 as *mut clistiter;
    let mut mvbox_folder: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut sentbox_folder: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut fallback_folder: *mut libc::c_char = 0 as *mut libc::c_char;
    if !(imap.is_null() || (*imap).etpan.is_null()) {
        dc_log_info(
            context,
            0i32,
            b"Configuring IMAP-folders.\x00" as *const u8 as *const libc::c_char,
        );
        folder_list = list_folders(imap);
        fallback_folder = dc_mprintf(
            b"INBOX%c%s\x00" as *const u8 as *const libc::c_char,
            (*imap).imap_delimiter as libc::c_int,
            b"DeltaChat\x00" as *const u8 as *const libc::c_char,
        );
        iter = (*folder_list).first;
        while !iter.is_null() {
            let mut folder: *mut dc_imapfolder_t = (if !iter.is_null() {
                (*iter).data
            } else {
                0 as *mut libc::c_void
            }) as *mut dc_imapfolder_t;
            if strcmp(
                (*folder).name_utf8,
                b"DeltaChat\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
                || strcmp((*folder).name_utf8, fallback_folder) == 0i32
            {
                if mvbox_folder.is_null() {
                    mvbox_folder = dc_strdup((*folder).name_to_select)
                }
            }
            if (*folder).meaning == 1i32 {
                if sentbox_folder.is_null() {
                    sentbox_folder = dc_strdup((*folder).name_to_select)
                }
            }
            iter = if !iter.is_null() {
                (*iter).next
            } else {
                0 as *mut clistcell_s
            }
        }
        if mvbox_folder.is_null() && 0 != flags & 0x1i32 {
            dc_log_info(
                context,
                0i32,
                b"Creating MVBOX-folder \"%s\"...\x00" as *const u8 as *const libc::c_char,
                b"DeltaChat\x00" as *const u8 as *const libc::c_char,
            );
            let mut r: libc::c_int = mailimap_create(
                (*imap).etpan,
                b"DeltaChat\x00" as *const u8 as *const libc::c_char,
            );
            if 0 != dc_imap_is_error(imap, r) {
                dc_log_warning(
                    context,
                    0i32,
                    b"Cannot create MVBOX-folder, using trying INBOX subfolder.\x00" as *const u8
                        as *const libc::c_char,
                );
                r = mailimap_create((*imap).etpan, fallback_folder);
                if 0 != dc_imap_is_error(imap, r) {
                    dc_log_warning(
                        context,
                        0i32,
                        b"Cannot create MVBOX-folder.\x00" as *const u8 as *const libc::c_char,
                    );
                } else {
                    mvbox_folder = dc_strdup(fallback_folder);
                    dc_log_info(
                        context,
                        0i32,
                        b"MVBOX-folder created as INBOX subfolder.\x00" as *const u8
                            as *const libc::c_char,
                    );
                }
            } else {
                mvbox_folder = dc_strdup(b"DeltaChat\x00" as *const u8 as *const libc::c_char);
                dc_log_info(
                    context,
                    0i32,
                    b"MVBOX-folder created.\x00" as *const u8 as *const libc::c_char,
                );
            }
            mailimap_subscribe((*imap).etpan, mvbox_folder);
        }
        dc_sqlite3_set_config_int(
            (*context).sql,
            b"folders_configured\x00" as *const u8 as *const libc::c_char,
            3i32,
        );
        dc_sqlite3_set_config(
            (*context).sql,
            b"configured_mvbox_folder\x00" as *const u8 as *const libc::c_char,
            mvbox_folder,
        );
        dc_sqlite3_set_config(
            (*context).sql,
            b"configured_sentbox_folder\x00" as *const u8 as *const libc::c_char,
            sentbox_folder,
        );
    }
    free_folders(folder_list);
    free(mvbox_folder as *mut libc::c_void);
    free(fallback_folder as *mut libc::c_void);
}
unsafe extern "C" fn free_folders(mut folders: *mut clist) {
    if !folders.is_null() {
        let mut iter1: *mut clistiter = 0 as *mut clistiter;
        iter1 = (*folders).first;
        while !iter1.is_null() {
            let mut ret_folder: *mut dc_imapfolder_t = (if !iter1.is_null() {
                (*iter1).data
            } else {
                0 as *mut libc::c_void
            }) as *mut dc_imapfolder_t;
            free((*ret_folder).name_to_select as *mut libc::c_void);
            free((*ret_folder).name_utf8 as *mut libc::c_void);
            free(ret_folder as *mut libc::c_void);
            iter1 = if !iter1.is_null() {
                (*iter1).next
            } else {
                0 as *mut clistcell_s
            }
        }
        clist_free(folders);
    };
}
unsafe extern "C" fn list_folders(mut imap: *mut dc_imap_t) -> *mut clist {
    let mut imap_list: *mut clist = 0 as *mut clist;
    let mut iter1: *mut clistiter = 0 as *mut clistiter;
    let mut ret_list: *mut clist = clist_new();
    let mut r: libc::c_int = 0i32;
    let mut xlist_works: libc::c_int = 0i32;
    if !(imap.is_null() || (*imap).etpan.is_null()) {
        if 0 != (*imap).has_xlist {
            r = mailimap_xlist(
                (*imap).etpan,
                b"\x00" as *const u8 as *const libc::c_char,
                b"*\x00" as *const u8 as *const libc::c_char,
                &mut imap_list,
            )
        } else {
            r = mailimap_list(
                (*imap).etpan,
                b"\x00" as *const u8 as *const libc::c_char,
                b"*\x00" as *const u8 as *const libc::c_char,
                &mut imap_list,
            )
        }
        if 0 != dc_imap_is_error(imap, r) || imap_list.is_null() {
            imap_list = 0 as *mut clist;
            dc_log_warning(
                (*imap).context,
                0i32,
                b"Cannot get folder list.\x00" as *const u8 as *const libc::c_char,
            );
        } else if (*imap_list).count <= 0i32 {
            dc_log_warning(
                (*imap).context,
                0i32,
                b"Folder list is empty.\x00" as *const u8 as *const libc::c_char,
            );
        } else {
            (*imap).imap_delimiter = '.' as i32 as libc::c_char;
            iter1 = (*imap_list).first;
            while !iter1.is_null() {
                let mut imap_folder: *mut mailimap_mailbox_list = (if !iter1.is_null() {
                    (*iter1).data
                } else {
                    0 as *mut libc::c_void
                })
                    as *mut mailimap_mailbox_list;
                if 0 != (*imap_folder).mb_delimiter {
                    (*imap).imap_delimiter = (*imap_folder).mb_delimiter
                }
                let mut ret_folder: *mut dc_imapfolder_t = calloc(
                    1i32 as libc::c_ulong,
                    ::std::mem::size_of::<dc_imapfolder_t>() as libc::c_ulong,
                )
                    as *mut dc_imapfolder_t;
                if strcasecmp(
                    (*imap_folder).mb_name,
                    b"INBOX\x00" as *const u8 as *const libc::c_char,
                ) == 0i32
                {
                    (*ret_folder).name_to_select =
                        dc_strdup(b"INBOX\x00" as *const u8 as *const libc::c_char)
                } else {
                    (*ret_folder).name_to_select = dc_strdup((*imap_folder).mb_name)
                }
                (*ret_folder).name_utf8 = dc_decode_modified_utf7((*imap_folder).mb_name, 0i32);
                (*ret_folder).meaning = get_folder_meaning((*imap_folder).mb_flag);
                if (*ret_folder).meaning == 2i32 || (*ret_folder).meaning == 1i32 {
                    xlist_works = 1i32
                }
                clist_insert_after(ret_list, (*ret_list).last, ret_folder as *mut libc::c_void);
                iter1 = if !iter1.is_null() {
                    (*iter1).next
                } else {
                    0 as *mut clistcell_s
                }
            }
            if 0 == xlist_works {
                iter1 = (*ret_list).first;
                while !iter1.is_null() {
                    let mut ret_folder_0: *mut dc_imapfolder_t = (if !iter1.is_null() {
                        (*iter1).data
                    } else {
                        0 as *mut libc::c_void
                    })
                        as *mut dc_imapfolder_t;
                    (*ret_folder_0).meaning = get_folder_meaning_by_name((*ret_folder_0).name_utf8);
                    iter1 = if !iter1.is_null() {
                        (*iter1).next
                    } else {
                        0 as *mut clistcell_s
                    }
                }
            }
        }
    }
    if !imap_list.is_null() {
        mailimap_list_result_free(imap_list);
    }
    return ret_list;
}
unsafe extern "C" fn get_folder_meaning_by_name(
    mut folder_name: *const libc::c_char,
) -> libc::c_int {
    // try to get the folder meaning by the name of the folder.
    // only used if the server does not support XLIST.
    let mut ret_meaning: libc::c_int = 0i32;
    // TODO: lots languages missing - maybe there is a list somewhere on other MUAs?
    // however, if we fail to find out the sent-folder,
    // only watching this folder is not working. at least, this is no show stopper.
    // CAVE: if possible, take care not to add a name here that is "sent" in one language
    // but sth. different in others - a hard job.
    static mut sent_names: *const libc::c_char =
        b",sent,sent objects,gesendet,\x00" as *const u8 as *const libc::c_char;
    let mut lower: *mut libc::c_char =
        dc_mprintf(b",%s,\x00" as *const u8 as *const libc::c_char, folder_name);
    dc_strlower_in_place(lower);
    if !strstr(sent_names, lower).is_null() {
        ret_meaning = 1i32
    }
    free(lower as *mut libc::c_void);
    return ret_meaning;
}
unsafe extern "C" fn get_folder_meaning(mut flags: *mut mailimap_mbx_list_flags) -> libc::c_int {
    let mut ret_meaning: libc::c_int = 0i32;
    if !flags.is_null() {
        let mut iter2: *mut clistiter = 0 as *mut clistiter;
        iter2 = (*(*flags).mbf_oflags).first;
        while !iter2.is_null() {
            let mut oflag: *mut mailimap_mbx_list_oflag = (if !iter2.is_null() {
                (*iter2).data
            } else {
                0 as *mut libc::c_void
            })
                as *mut mailimap_mbx_list_oflag;
            match (*oflag).of_type {
                2 => {
                    if strcasecmp(
                        (*oflag).of_flag_ext,
                        b"spam\x00" as *const u8 as *const libc::c_char,
                    ) == 0i32
                        || strcasecmp(
                            (*oflag).of_flag_ext,
                            b"trash\x00" as *const u8 as *const libc::c_char,
                        ) == 0i32
                        || strcasecmp(
                            (*oflag).of_flag_ext,
                            b"drafts\x00" as *const u8 as *const libc::c_char,
                        ) == 0i32
                        || strcasecmp(
                            (*oflag).of_flag_ext,
                            b"junk\x00" as *const u8 as *const libc::c_char,
                        ) == 0i32
                    {
                        ret_meaning = 2i32
                    } else if strcasecmp(
                        (*oflag).of_flag_ext,
                        b"sent\x00" as *const u8 as *const libc::c_char,
                    ) == 0i32
                    {
                        ret_meaning = 1i32
                    }
                }
                _ => {}
            }
            iter2 = if !iter2.is_null() {
                (*iter2).next
            } else {
                0 as *mut clistcell_s
            }
        }
    }
    return ret_meaning;
}
unsafe extern "C" fn moz_autoconfigure(
    mut context: *mut dc_context_t,
    mut url: *const libc::c_char,
    mut param_in: *const dc_loginparam_t,
) -> *mut dc_loginparam_t {
    let mut p: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut saxparser: dc_saxparser_t = dc_saxparser_t {
        starttag_cb: None,
        endtag_cb: None,
        text_cb: None,
        userdata: 0 as *mut libc::c_void,
    };
    let mut xml_raw: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut moz_ac: moz_autoconfigure_t = moz_autoconfigure_t {
        in_0: 0 as *const dc_loginparam_t,
        in_emaildomain: 0 as *mut libc::c_char,
        in_emaillocalpart: 0 as *mut libc::c_char,
        out: 0 as *mut dc_loginparam_t,
        out_imap_set: 0,
        out_smtp_set: 0,
        tag_server: 0,
        tag_config: 0,
    };
    memset(
        &mut moz_ac as *mut moz_autoconfigure_t as *mut libc::c_void,
        0i32,
        ::std::mem::size_of::<moz_autoconfigure_t>() as libc::c_ulong,
    );
    xml_raw = read_autoconf_file(context, url);
    if !xml_raw.is_null() {
        moz_ac.in_0 = param_in;
        moz_ac.in_emaillocalpart = dc_strdup((*param_in).addr);
        p = strchr(moz_ac.in_emaillocalpart, '@' as i32);
        if !p.is_null() {
            *p = 0i32 as libc::c_char;
            moz_ac.in_emaildomain = dc_strdup(p.offset(1isize));
            moz_ac.out = dc_loginparam_new();
            saxparser = dc_saxparser_t {
                starttag_cb: None,
                endtag_cb: None,
                text_cb: None,
                userdata: 0 as *mut libc::c_void,
            };
            dc_saxparser_init(
                &mut saxparser,
                &mut moz_ac as *mut moz_autoconfigure_t as *mut libc::c_void,
            );
            dc_saxparser_set_tag_handler(
                &mut saxparser,
                Some(moz_autoconfigure_starttag_cb),
                Some(moz_autoconfigure_endtag_cb),
            );
            dc_saxparser_set_text_handler(&mut saxparser, Some(moz_autoconfigure_text_cb));
            dc_saxparser_parse(&mut saxparser, xml_raw);
            if (*moz_ac.out).mail_server.is_null()
                || (*moz_ac.out).mail_port as libc::c_int == 0i32
                || (*moz_ac.out).send_server.is_null()
                || (*moz_ac.out).send_port == 0i32
            {
                let mut r: *mut libc::c_char = dc_loginparam_get_readable(moz_ac.out);
                dc_log_warning(
                    context,
                    0i32,
                    b"Bad or incomplete autoconfig: %s\x00" as *const u8 as *const libc::c_char,
                    r,
                );
                free(r as *mut libc::c_void);
                dc_loginparam_unref(moz_ac.out);
                moz_ac.out = 0 as *mut dc_loginparam_t
            }
        }
    }
    free(xml_raw as *mut libc::c_void);
    free(moz_ac.in_emaildomain as *mut libc::c_void);
    free(moz_ac.in_emaillocalpart as *mut libc::c_void);
    return moz_ac.out;
}
unsafe extern "C" fn moz_autoconfigure_text_cb(
    mut userdata: *mut libc::c_void,
    mut text: *const libc::c_char,
    mut len: libc::c_int,
) {
    let mut moz_ac: *mut moz_autoconfigure_t = userdata as *mut moz_autoconfigure_t;
    let mut val: *mut libc::c_char = dc_strdup(text);
    dc_trim(val);
    dc_str_replace(
        &mut val,
        b"%EMAILADDRESS%\x00" as *const u8 as *const libc::c_char,
        (*(*moz_ac).in_0).addr,
    );
    dc_str_replace(
        &mut val,
        b"%EMAILLOCALPART%\x00" as *const u8 as *const libc::c_char,
        (*moz_ac).in_emaillocalpart,
    );
    dc_str_replace(
        &mut val,
        b"%EMAILDOMAIN%\x00" as *const u8 as *const libc::c_char,
        (*moz_ac).in_emaildomain,
    );
    if (*moz_ac).tag_server == 1i32 {
        match (*moz_ac).tag_config {
            10 => {
                free((*(*moz_ac).out).mail_server as *mut libc::c_void);
                (*(*moz_ac).out).mail_server = val;
                val = 0 as *mut libc::c_char
            }
            11 => (*(*moz_ac).out).mail_port = atoi(val) as uint16_t,
            12 => {
                free((*(*moz_ac).out).mail_user as *mut libc::c_void);
                (*(*moz_ac).out).mail_user = val;
                val = 0 as *mut libc::c_char
            }
            13 => {
                if strcasecmp(val, b"ssl\x00" as *const u8 as *const libc::c_char) == 0i32 {
                    (*(*moz_ac).out).server_flags |= 0x200i32
                }
                if strcasecmp(val, b"starttls\x00" as *const u8 as *const libc::c_char) == 0i32 {
                    (*(*moz_ac).out).server_flags |= 0x100i32
                }
                if strcasecmp(val, b"plain\x00" as *const u8 as *const libc::c_char) == 0i32 {
                    (*(*moz_ac).out).server_flags |= 0x400i32
                }
            }
            _ => {}
        }
    } else if (*moz_ac).tag_server == 2i32 {
        match (*moz_ac).tag_config {
            10 => {
                free((*(*moz_ac).out).send_server as *mut libc::c_void);
                (*(*moz_ac).out).send_server = val;
                val = 0 as *mut libc::c_char
            }
            11 => (*(*moz_ac).out).send_port = atoi(val),
            12 => {
                free((*(*moz_ac).out).send_user as *mut libc::c_void);
                (*(*moz_ac).out).send_user = val;
                val = 0 as *mut libc::c_char
            }
            13 => {
                if strcasecmp(val, b"ssl\x00" as *const u8 as *const libc::c_char) == 0i32 {
                    (*(*moz_ac).out).server_flags |= 0x20000i32
                }
                if strcasecmp(val, b"starttls\x00" as *const u8 as *const libc::c_char) == 0i32 {
                    (*(*moz_ac).out).server_flags |= 0x10000i32
                }
                if strcasecmp(val, b"plain\x00" as *const u8 as *const libc::c_char) == 0i32 {
                    (*(*moz_ac).out).server_flags |= 0x40000i32
                }
            }
            _ => {}
        }
    }
    free(val as *mut libc::c_void);
}
unsafe extern "C" fn moz_autoconfigure_endtag_cb(
    mut userdata: *mut libc::c_void,
    mut tag: *const libc::c_char,
) {
    let mut moz_ac: *mut moz_autoconfigure_t = userdata as *mut moz_autoconfigure_t;
    if strcmp(
        tag,
        b"incomingserver\x00" as *const u8 as *const libc::c_char,
    ) == 0i32
    {
        (*moz_ac).tag_server = 0i32;
        (*moz_ac).tag_config = 0i32;
        (*moz_ac).out_imap_set = 1i32
    } else if strcmp(
        tag,
        b"outgoingserver\x00" as *const u8 as *const libc::c_char,
    ) == 0i32
    {
        (*moz_ac).tag_server = 0i32;
        (*moz_ac).tag_config = 0i32;
        (*moz_ac).out_smtp_set = 1i32
    } else {
        (*moz_ac).tag_config = 0i32
    };
}
unsafe extern "C" fn moz_autoconfigure_starttag_cb(
    mut userdata: *mut libc::c_void,
    mut tag: *const libc::c_char,
    mut attr: *mut *mut libc::c_char,
) {
    let mut moz_ac: *mut moz_autoconfigure_t = userdata as *mut moz_autoconfigure_t;
    let mut p1: *const libc::c_char = 0 as *const libc::c_char;
    if strcmp(
        tag,
        b"incomingserver\x00" as *const u8 as *const libc::c_char,
    ) == 0i32
    {
        (*moz_ac).tag_server = if (*moz_ac).out_imap_set == 0i32
            && {
                p1 = dc_attr_find(attr, b"type\x00" as *const u8 as *const libc::c_char);
                !p1.is_null()
            }
            && strcasecmp(p1, b"imap\x00" as *const u8 as *const libc::c_char) == 0i32
        {
            1i32
        } else {
            0i32
        };
        (*moz_ac).tag_config = 0i32
    } else if strcmp(
        tag,
        b"outgoingserver\x00" as *const u8 as *const libc::c_char,
    ) == 0i32
    {
        (*moz_ac).tag_server = if (*moz_ac).out_smtp_set == 0i32 {
            2i32
        } else {
            0i32
        };
        (*moz_ac).tag_config = 0i32
    } else if strcmp(tag, b"hostname\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*moz_ac).tag_config = 10i32
    } else if strcmp(tag, b"port\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*moz_ac).tag_config = 11i32
    } else if strcmp(tag, b"sockettype\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*moz_ac).tag_config = 13i32
    } else if strcmp(tag, b"username\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*moz_ac).tag_config = 12i32
    };
}
unsafe extern "C" fn read_autoconf_file(
    mut context: *mut dc_context_t,
    mut url: *const libc::c_char,
) -> *mut libc::c_char {
    let mut filecontent: *mut libc::c_char = 0 as *mut libc::c_char;
    dc_log_info(
        context,
        0i32,
        b"Testing %s ...\x00" as *const u8 as *const libc::c_char,
        url,
    );
    filecontent = (*context).cb.expect("non-null function pointer")(
        context,
        2100i32,
        url as uintptr_t,
        0i32 as uintptr_t,
    ) as *mut libc::c_char;
    if filecontent.is_null() || *filecontent.offset(0isize) as libc::c_int == 0i32 {
        free(filecontent as *mut libc::c_void);
        dc_log_info(
            context,
            0i32,
            b"Can\'t read file.\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as *mut libc::c_char;
    }
    return filecontent;
}
unsafe extern "C" fn outlk_autodiscover(
    mut context: *mut dc_context_t,
    mut url__: *const libc::c_char,
    mut param_in: *const dc_loginparam_t,
) -> *mut dc_loginparam_t {
    let mut current_block: u64;
    let mut xml_raw: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut url: *mut libc::c_char = dc_strdup(url__);
    let mut outlk_ad: outlk_autodiscover_t = outlk_autodiscover_t {
        in_0: 0 as *const dc_loginparam_t,
        out: 0 as *mut dc_loginparam_t,
        out_imap_set: 0,
        out_smtp_set: 0,
        tag_config: 0,
        config: [0 as *mut libc::c_char; 6],
        redirect: 0 as *mut libc::c_char,
    };
    let mut i: libc::c_int = 0;
    i = 0i32;
    loop {
        if !(i < 10i32) {
            current_block = 11584701595673473500;
            break;
        }
        memset(
            &mut outlk_ad as *mut outlk_autodiscover_t as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<outlk_autodiscover_t>() as libc::c_ulong,
        );
        xml_raw = read_autoconf_file(context, url);
        if xml_raw.is_null() {
            current_block = 3070887585260837332;
            break;
        }
        outlk_ad.in_0 = param_in;
        outlk_ad.out = dc_loginparam_new();
        let mut saxparser: dc_saxparser_t = dc_saxparser_t {
            starttag_cb: None,
            endtag_cb: None,
            text_cb: None,
            userdata: 0 as *mut libc::c_void,
        };
        dc_saxparser_init(
            &mut saxparser,
            &mut outlk_ad as *mut outlk_autodiscover_t as *mut libc::c_void,
        );
        dc_saxparser_set_tag_handler(
            &mut saxparser,
            Some(outlk_autodiscover_starttag_cb),
            Some(outlk_autodiscover_endtag_cb),
        );
        dc_saxparser_set_text_handler(&mut saxparser, Some(outlk_autodiscover_text_cb));
        dc_saxparser_parse(&mut saxparser, xml_raw);
        if !(!outlk_ad.config[5usize].is_null()
            && 0 != *outlk_ad.config[5usize].offset(0isize) as libc::c_int)
        {
            current_block = 11584701595673473500;
            break;
        }
        free(url as *mut libc::c_void);
        url = dc_strdup(outlk_ad.config[5usize]);
        dc_loginparam_unref(outlk_ad.out);
        outlk_clean_config(&mut outlk_ad);
        free(xml_raw as *mut libc::c_void);
        xml_raw = 0 as *mut libc::c_char;
        i += 1
    }
    match current_block {
        11584701595673473500 => {
            if (*outlk_ad.out).mail_server.is_null()
                || (*outlk_ad.out).mail_port as libc::c_int == 0i32
                || (*outlk_ad.out).send_server.is_null()
                || (*outlk_ad.out).send_port == 0i32
            {
                let mut r: *mut libc::c_char = dc_loginparam_get_readable(outlk_ad.out);
                dc_log_warning(
                    context,
                    0i32,
                    b"Bad or incomplete autoconfig: %s\x00" as *const u8 as *const libc::c_char,
                    r,
                );
                free(r as *mut libc::c_void);
                dc_loginparam_unref(outlk_ad.out);
                outlk_ad.out = 0 as *mut dc_loginparam_t
            }
        }
        _ => {}
    }
    free(url as *mut libc::c_void);
    free(xml_raw as *mut libc::c_void);
    outlk_clean_config(&mut outlk_ad);
    return outlk_ad.out;
}
unsafe extern "C" fn outlk_clean_config(mut outlk_ad: *mut outlk_autodiscover_t) {
    let mut i: libc::c_int = 0;
    i = 0i32;
    while i < 6i32 {
        free((*outlk_ad).config[i as usize] as *mut libc::c_void);
        (*outlk_ad).config[i as usize] = 0 as *mut libc::c_char;
        i += 1
    }
}
unsafe extern "C" fn outlk_autodiscover_text_cb(
    mut userdata: *mut libc::c_void,
    mut text: *const libc::c_char,
    mut len: libc::c_int,
) {
    let mut outlk_ad: *mut outlk_autodiscover_t = userdata as *mut outlk_autodiscover_t;
    let mut val: *mut libc::c_char = dc_strdup(text);
    dc_trim(val);
    free((*outlk_ad).config[(*outlk_ad).tag_config as usize] as *mut libc::c_void);
    (*outlk_ad).config[(*outlk_ad).tag_config as usize] = val;
}
unsafe extern "C" fn outlk_autodiscover_endtag_cb(
    mut userdata: *mut libc::c_void,
    mut tag: *const libc::c_char,
) {
    let mut outlk_ad: *mut outlk_autodiscover_t = userdata as *mut outlk_autodiscover_t;
    if strcmp(tag, b"protocol\x00" as *const u8 as *const libc::c_char) == 0i32 {
        if !(*outlk_ad).config[1usize].is_null() {
            let mut port: libc::c_int = dc_atoi_null_is_0((*outlk_ad).config[3usize]);
            let mut ssl_on: libc::c_int = (!(*outlk_ad).config[4usize].is_null()
                && strcasecmp(
                    (*outlk_ad).config[4usize],
                    b"on\x00" as *const u8 as *const libc::c_char,
                ) == 0i32) as libc::c_int;
            let mut ssl_off: libc::c_int = (!(*outlk_ad).config[4usize].is_null()
                && strcasecmp(
                    (*outlk_ad).config[4usize],
                    b"off\x00" as *const u8 as *const libc::c_char,
                ) == 0i32) as libc::c_int;
            if strcasecmp(
                (*outlk_ad).config[1usize],
                b"imap\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
                && (*outlk_ad).out_imap_set == 0i32
            {
                (*(*outlk_ad).out).mail_server = dc_strdup_keep_null((*outlk_ad).config[2usize]);
                (*(*outlk_ad).out).mail_port = port as uint16_t;
                if 0 != ssl_on {
                    (*(*outlk_ad).out).server_flags |= 0x200i32
                } else if 0 != ssl_off {
                    (*(*outlk_ad).out).server_flags |= 0x400i32
                }
                (*outlk_ad).out_imap_set = 1i32
            } else if strcasecmp(
                (*outlk_ad).config[1usize],
                b"smtp\x00" as *const u8 as *const libc::c_char,
            ) == 0i32
                && (*outlk_ad).out_smtp_set == 0i32
            {
                (*(*outlk_ad).out).send_server = dc_strdup_keep_null((*outlk_ad).config[2usize]);
                (*(*outlk_ad).out).send_port = port;
                if 0 != ssl_on {
                    (*(*outlk_ad).out).server_flags |= 0x20000i32
                } else if 0 != ssl_off {
                    (*(*outlk_ad).out).server_flags |= 0x40000i32
                }
                (*outlk_ad).out_smtp_set = 1i32
            }
        }
        outlk_clean_config(outlk_ad);
    }
    (*outlk_ad).tag_config = 0i32;
}
unsafe extern "C" fn outlk_autodiscover_starttag_cb(
    mut userdata: *mut libc::c_void,
    mut tag: *const libc::c_char,
    mut attr: *mut *mut libc::c_char,
) {
    let mut outlk_ad: *mut outlk_autodiscover_t = userdata as *mut outlk_autodiscover_t;
    if strcmp(tag, b"protocol\x00" as *const u8 as *const libc::c_char) == 0i32 {
        outlk_clean_config(outlk_ad);
    } else if strcmp(tag, b"type\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*outlk_ad).tag_config = 1i32
    } else if strcmp(tag, b"server\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*outlk_ad).tag_config = 2i32
    } else if strcmp(tag, b"port\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*outlk_ad).tag_config = 3i32
    } else if strcmp(tag, b"ssl\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*outlk_ad).tag_config = 4i32
    } else if strcmp(tag, b"redirecturl\x00" as *const u8 as *const libc::c_char) == 0i32 {
        (*outlk_ad).tag_config = 5i32
    };
}
#[no_mangle]
pub unsafe extern "C" fn dc_alloc_ongoing(mut context: *mut dc_context_t) -> libc::c_int {
    if context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint {
        return 0i32;
    }
    if 0 != dc_has_ongoing(context) {
        dc_log_warning(
            context,
            0i32,
            b"There is already another ongoing process running.\x00" as *const u8
                as *const libc::c_char,
        );
        return 0i32;
    }
    (*context).ongoing_running = 1i32;
    (*context).shall_stop_ongoing = 0i32;
    return 1i32;
}
#[no_mangle]
pub unsafe extern "C" fn dc_connect_to_configured_imap(
    mut context: *mut dc_context_t,
    mut imap: *mut dc_imap_t,
) -> libc::c_int {
    let mut ret_connected: libc::c_int = 0i32;
    let mut param: *mut dc_loginparam_t = dc_loginparam_new();
    if context.is_null() || (*context).magic != 0x11a11807i32 as libc::c_uint || imap.is_null() {
        dc_log_warning(
            (*imap).context,
            0i32,
            b"Cannot connect to IMAP: Bad parameters.\x00" as *const u8 as *const libc::c_char,
        );
    } else if 0 != dc_imap_is_connected(imap) {
        ret_connected = 1i32
    } else if dc_sqlite3_get_config_int(
        (*(*imap).context).sql,
        b"configured\x00" as *const u8 as *const libc::c_char,
        0i32,
    ) == 0i32
    {
        dc_log_warning(
            (*imap).context,
            0i32,
            b"Not configured, cannot connect.\x00" as *const u8 as *const libc::c_char,
        );
    } else {
        dc_loginparam_read(
            param,
            (*(*imap).context).sql,
            b"configured_\x00" as *const u8 as *const libc::c_char,
        );
        /*the trailing underscore is correct*/
        if !(0 == dc_imap_connect(imap, param)) {
            ret_connected = 2i32
        }
    }
    dc_loginparam_unref(param);
    return ret_connected;
}