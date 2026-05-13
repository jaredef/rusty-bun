// Π2.6.basket-tz — IANA timezone substrate via libc.
//
// Exposes one primitive: `local_time_at(utc_ms, tz_name)` →
// (year, month, day, hour, minute, second, offset_minutes, abbr).
// Implementation sets the TZ environment variable, calls
// localtime_r, then restores TZ. Linux's tzdata provides the IANA
// database; macOS has it too via the same libc surface. This is
// the cheapest substrate widening that unlocks date-fns-tz,
// dayjs/plugin/timezone, luxon's full TZ surface, moment-timezone
// fallback paths, and every i18n stack that exercises
// Intl.DateTimeFormat with the timeZone option.
//
// Per-thread serialization: localtime_r is reentrant but TZ env
// mutation is process-global. We guard with a mutex so concurrent
// calls don't race on the env var.

#[cfg(unix)]
pub mod tz {
    use std::sync::Mutex;

    static TZ_LOCK: Mutex<()> = Mutex::new(());

    extern "C" {
        fn tzset();
    }

    pub struct LocalTime {
        pub year: i32,
        pub month: u32,   // 1..12
        pub day: u32,
        pub hour: u32,
        pub minute: u32,
        pub second: u32,
        pub offset_minutes: i32,
        pub abbr: String,
        pub weekday: u32,  // 0=Sunday..6=Saturday
    }

    pub fn local_time_at(utc_ms: i64, tz_name: &str) -> Result<LocalTime, String> {
        let _guard = TZ_LOCK.lock().map_err(|e| e.to_string())?;
        let prev_tz = std::env::var("TZ").ok();
        unsafe {
            let key = std::ffi::CString::new("TZ").unwrap();
            let val = std::ffi::CString::new(tz_name).unwrap();
            libc::setenv(key.as_ptr(), val.as_ptr(), 1);
            tzset();
        }
        let secs = utc_ms.div_euclid(1000);
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::localtime_r(&secs as *const i64, &mut tm) };
        // Restore prior TZ before returning to caller (panic-safe-ish).
        unsafe {
            let key = std::ffi::CString::new("TZ").unwrap();
            match prev_tz {
                Some(v) => {
                    let cv = std::ffi::CString::new(v).unwrap();
                    libc::setenv(key.as_ptr(), cv.as_ptr(), 1);
                }
                None => {
                    libc::unsetenv(key.as_ptr());
                }
            }
            tzset();
        }
        if rc.is_null() {
            return Err(format!("localtime_r failed for tz={}", tz_name));
        }
        let abbr = unsafe {
            if tm.tm_zone.is_null() { String::new() }
            else { std::ffi::CStr::from_ptr(tm.tm_zone).to_string_lossy().into_owned() }
        };
        Ok(LocalTime {
            year: tm.tm_year + 1900,
            month: (tm.tm_mon + 1) as u32,
            day: tm.tm_mday as u32,
            hour: tm.tm_hour as u32,
            minute: tm.tm_min as u32,
            second: tm.tm_sec as u32,
            offset_minutes: (tm.tm_gmtoff / 60) as i32,
            abbr,
            weekday: tm.tm_wday as u32,
        })
    }
}

#[cfg(not(unix))]
pub mod tz {
    pub struct LocalTime {
        pub year: i32, pub month: u32, pub day: u32,
        pub hour: u32, pub minute: u32, pub second: u32,
        pub offset_minutes: i32, pub abbr: String, pub weekday: u32,
    }
    pub fn local_time_at(_utc_ms: i64, _tz_name: &str) -> Result<LocalTime, String> {
        Err("tz substrate only on unix".to_string())
    }
}
