mod feature;
mod store;
mod time;
pub(crate) mod tools;

pub(crate) use feature::Feature;

/// Job ids this feature owns: the sync job and the per-event create jobs.
pub(crate) const JOB_PREFIXES: &[&str] = &[feature::SYNC_JOB, feature::CREATE_JOB_PREFIX];
