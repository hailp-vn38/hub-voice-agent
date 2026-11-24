mod feature;
pub(crate) mod tools;

pub(crate) use feature::Feature;

/// Job ids are this prefix followed by the entity id.
pub(crate) const JOB_PREFIX: &str = "home:";
