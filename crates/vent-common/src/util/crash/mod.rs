use chrono::{DateTime, Local};
use rfd::MessageLevel;
use std::fs::File;
use std::io::Write;
use std::panic::{self, PanicInfo};
use sysinfo::{System, SystemExt};

#[inline]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(panic_handler));
}

fn panic_handler(pi: &PanicInfo) {
    log::warn!("Crashing...");
    log_crash(pi).expect("Failed to Log Crash File");
    show_error_dialog(pi);
}

fn log_crash(pi: &PanicInfo) -> std::io::Result<()> {
    let timestamp: DateTime<Local> = Local::now();

    let mut sys = System::new_all();
    sys.refresh_all();
    // let drivers = get_driver_information(); // Implement this function to retrieve driver information

    // Generate log file name based on timestamp
    let log_file_name = format!("crash/crash_log_{}.log", timestamp.format("%Y%m%d%H%M%S"));

    // Create and write crash information to the log file

    let mut f = File::create(log_file_name)?;
    let mut perms = f.metadata()?.permissions();
    perms.set_readonly(true);
    f.set_permissions(perms)?;

    writeln!(&mut f, "--Crash Log--")?;
    writeln!(
        &mut f,
        "System kernel version:   {:?}",
        sys.kernel_version()
    )?;
    writeln!(&mut f, "System OS version:       {:?}", sys.os_version())?;
    writeln!(&mut f, "System host name:        {:?}", sys.host_name())?;

    writeln!(
        &mut f,
        "System Core Count:             {:?}",
        sys.physical_core_count()
    )?;
    writeln!(
        &mut f,
        "System Total Memory:             {:?}",
        sys.total_memory()
    )?;

    //   writeln!(&mut f, "Driver Information: {}", drivers)?;
    writeln!(&mut f, "Crash Info:              {:?}", format!("{:?}", pi))?;
    Ok(())
}

fn show_error_dialog(pi: &PanicInfo) {
    rfd::MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("Application Error")
        .set_description(format!("{:?}", pi).as_str())
        .show();
}
