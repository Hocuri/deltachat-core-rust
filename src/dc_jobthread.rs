use std::sync::{Arc, Condvar, Mutex};

use crate::context::Context;
use crate::dc_configure::*;
use crate::dc_log::*;
use crate::dc_sqlite3::*;
use crate::dc_tools::*;
use crate::imap::Imap;
use crate::x::*;
use std::ffi::CString;

#[repr(C)]
pub struct dc_jobthread_t {
    pub name: &'static str,
    pub folder_config_name: &'static str,
    pub imap: Imap,
    pub state: Arc<(Mutex<JobState>, Condvar)>,
}

pub fn dc_jobthread_init(
    name: &'static str,
    folder_config_name: &'static str,
    imap: Imap,
) -> dc_jobthread_t {
    dc_jobthread_t {
        name,
        folder_config_name,
        imap,
        state: Arc::new((Mutex::new(Default::default()), Condvar::new())),
    }
}

#[derive(Debug, Default)]
pub struct JobState {
    idle: bool,
    jobs_needed: i32,
    suspended: i32,
    using_handle: i32,
}

pub unsafe fn dc_jobthread_suspend(
    context: &Context,
    jobthread: &dc_jobthread_t,
    suspend: libc::c_int,
) {
    if 0 != suspend {
        dc_log_info(
            context,
            0i32,
            b"Suspending %s-thread.\x00" as *const u8 as *const libc::c_char,
            &jobthread.name,
        );

        {
            jobthread.state.0.lock().unwrap().suspended = 1;
        }
        dc_jobthread_interrupt_idle(context, jobthread);
        loop {
            let using_handle = jobthread.state.0.lock().unwrap().using_handle;
            if using_handle == 0 {
                return;
            }
            std::thread::sleep(std::time::Duration::from_micros(300 * 1000));
        }
    } else {
        dc_log_info(
            context,
            0i32,
            b"Unsuspending %s-thread.\x00" as *const u8 as *const libc::c_char,
            &jobthread.name,
        );

        let &(ref lock, ref cvar) = &*jobthread.state.clone();
        let mut state = lock.lock().unwrap();

        state.suspended = 0;
        state.idle = true;
        cvar.notify_one();
    }
}

pub unsafe fn dc_jobthread_interrupt_idle(context: &Context, jobthread: &dc_jobthread_t) {
    {
        jobthread.state.0.lock().unwrap().jobs_needed = 1;
    }

    dc_log_info(
        context,
        0,
        b"Interrupting %s-IDLE...\x00" as *const u8 as *const libc::c_char,
        &jobthread.name,
    );

    jobthread.imap.interrupt_idle();

    let &(ref lock, ref cvar) = &*jobthread.state.clone();
    let mut state = lock.lock().unwrap();

    state.idle = true;
    cvar.notify_one();
}

pub unsafe fn dc_jobthread_fetch(
    context: &Context,
    jobthread: &mut dc_jobthread_t,
    use_network: libc::c_int,
) {
    let start;

    {
        let &(ref lock, _) = &*jobthread.state.clone();
        let mut state = lock.lock().unwrap();

        if 0 != state.suspended {
            return;
        }

        state.using_handle = 1;
    }

    if 0 != use_network {
        start = clock();
        if !(0 == connect_to_imap(context, jobthread)) {
            dc_log_info(
                context,
                0,
                b"%s-fetch started...\x00" as *const u8 as *const libc::c_char,
                &jobthread.name,
            );
            jobthread.imap.fetch(context);

            if jobthread.imap.should_reconnect() {
                dc_log_info(
                    context,
                    0i32,
                    b"%s-fetch aborted, starting over...\x00" as *const u8 as *const libc::c_char,
                    &jobthread.name,
                );
                jobthread.imap.fetch(context);
            }
            dc_log_info(
                context,
                0,
                b"%s-fetch done in %.0f ms.\x00" as *const u8 as *const libc::c_char,
                &jobthread.name,
                clock().wrapping_sub(start) as libc::c_double * 1000.0f64
                    / 1000000i32 as libc::c_double,
            );
        }
    }

    jobthread.state.0.lock().unwrap().using_handle = 0;
}

/* ******************************************************************************
 * the typical fetch, idle, interrupt-idle
 ******************************************************************************/

unsafe fn connect_to_imap(context: &Context, jobthread: &dc_jobthread_t) -> libc::c_int {
    let mut ret_connected: libc::c_int;
    let mut mvbox_name: *mut libc::c_char = 0 as *mut libc::c_char;

    if jobthread.imap.is_connected() {
        ret_connected = 1;
    } else {
        ret_connected = dc_connect_to_configured_imap(context, &jobthread.imap);
        if !(0 == ret_connected) {
            if dc_sqlite3_get_config_int(
                context,
                &context.sql,
                b"folders_configured\x00" as *const u8 as *const libc::c_char,
                0,
            ) < 3
            {
                jobthread.imap.configure_folders(context, 0x1);
            }
            mvbox_name = dc_sqlite3_get_config(
                context,
                &context.sql,
                CString::new(&jobthread.folder_config_name[..])
                    .unwrap()
                    .as_ptr(),
                0 as *const libc::c_char,
            );
            if mvbox_name.is_null() {
                jobthread.imap.disconnect(context);
                ret_connected = 0;
            } else {
                jobthread.imap.set_watch_folder(mvbox_name);
            }
        }
    }
    free(mvbox_name as *mut libc::c_void);

    ret_connected
}

pub unsafe fn dc_jobthread_idle(
    context: &Context,
    jobthread: &dc_jobthread_t,
    use_network: libc::c_int,
) {
    {
        let &(ref lock, ref cvar) = &*jobthread.state.clone();
        let mut state = lock.lock().unwrap();

        if 0 != state.jobs_needed {
            dc_log_info(
                context,
                0,
                b"%s-IDLE will not be started as it was interrupted while not ideling.\x00"
                    as *const u8 as *const libc::c_char,
                &jobthread.name,
            );
            state.jobs_needed = 0;
            return;
        }

        if 0 != state.suspended {
            while !state.idle {
                state = cvar.wait(state).unwrap();
            }
            state.idle = false;
            return;
        }

        state.using_handle = 1;

        if 0 == use_network {
            state.using_handle = 0;

            while !state.idle {
                state = cvar.wait(state).unwrap();
            }
            state.idle = false;
            return;
        }
    }

    connect_to_imap(context, jobthread);
    dc_log_info(
        context,
        0i32,
        b"%s-IDLE started...\x00" as *const u8 as *const libc::c_char,
        &jobthread.name,
    );
    jobthread.imap.idle(context);
    dc_log_info(
        context,
        0i32,
        b"%s-IDLE ended.\x00" as *const u8 as *const libc::c_char,
        &jobthread.name,
    );

    jobthread.state.0.lock().unwrap().using_handle = 0;
}
