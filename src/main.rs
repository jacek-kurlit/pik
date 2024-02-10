use uzers::{all_users, get_user_by_uid, User};

fn main() {
    let mut users: Vec<User> = unsafe { all_users() }.collect();
    users.sort_by_key(|a| a.uid());

    for user in users {
        println!(
            "User {} has name {}",
            user.uid(),
            user.name().to_string_lossy()
        );
    }
    // list_all_processes();
}

fn list_all_processes() {
    let tps = procfs::ticks_per_second();

    println!("{: >10} {: <8} {: >8} CMD", "PID", "TTY", "TIME");

    for p in procfs::process::all_processes().unwrap() {
        let prc = p.unwrap();
        let user = get_user_by_uid(prc.uid().unwrap_or(0)).unwrap();
        if let Ok(stat) = prc.stat() {
            // total_time is in seconds
            let total_time = (stat.utime + stat.stime) as f32 / (tps as f32);
            println!(
                "{: >15} {: >10} {: >8} {}",
                user.name().to_string_lossy().to_string(),
                stat.pid,
                total_time,
                stat.comm
            );
        }
    }
}
