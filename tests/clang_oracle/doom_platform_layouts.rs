use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn linux_timer_signal_struct_layouts_match_host_exit_code() {
    if !cfg!(target_os = "linux") {
        return;
    }

    // given
    let case = OracleCase {
        name: "linux_timer_signal_struct_layouts",
        source: "#define _GNU_SOURCE\n#include <sys/time.h>\n#include <signal.h>\nint main(void) { struct itimerval value; struct sigaction act; value.it_interval.tv_sec = 1; value.it_interval.tv_usec = 2; value.it_value.tv_sec = 3; value.it_value.tv_usec = 4; act.sa_flags = SA_RESTART; return sizeof(struct timeval) == 16 && sizeof(struct itimerval) == 32 && sizeof(struct sigaction) == 152 && value.it_interval.tv_sec == 1 && value.it_value.tv_usec == 4 && act.sa_flags == SA_RESTART ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
