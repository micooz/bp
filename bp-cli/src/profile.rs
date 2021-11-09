use jemalloc_ctl::{Access, AsName};
use jemallocator;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const PROF_ACTIVE: &'static [u8] = b"prof.active\0";
const PROF_DUMP: &'static [u8] = b"prof.dump\0";
const PROFILE_OUTPUT: &'static [u8] = b"profile.out\0";

pub fn set_prof_active(active: bool) {
    let name = PROF_ACTIVE.name();
    name.write(active).expect("Should succeed to set prof");
}

pub fn dump_profile() {
    let name = PROF_DUMP.name();
    name.write(PROFILE_OUTPUT).expect("Should succeed to dump profile");
}
