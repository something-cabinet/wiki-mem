use wm_engine::RECORD_SCHEMA_VERSION;

use super::page_version_helper::page_version;
use super::skip_reason_model::SkipReason;
use super::version_step_model::VersionStep;

pub const UPGRADE_STEPS: [VersionStep; 0] = [];

pub fn upgrade_record_body(
    body: &str,
    steps: &[VersionStep],
    target: u32,
) -> Result<(String, usize), SkipReason> {
    let mut current = body.to_owned();
    let mut applied = 0usize;
    loop {
        let version = page_version(&current);
        if version > target {
            return Err(SkipReason::UnsupportedVersion { found: version });
        }
        if version == target {
            return Ok((current, applied));
        }
        let step = steps
            .iter()
            .find(|step| step.from == version)
            .ok_or(SkipReason::MissingUpgradeStep { from: version })?;
        let next =
            (step.apply)(&current).map_err(|error| SkipReason::InvalidRecordBody { error })?;
        if page_version(&next) != version.saturating_add(1) {
            return Err(SkipReason::InvalidRecordBody {
                error: format!("upgrade step from version {version} did not advance the version"),
            });
        }
        current = next;
        applied = applied.saturating_add(1);
    }
}

pub fn current_record_version() -> u32 {
    RECORD_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bump_one_to_two(body: &str) -> Result<String, String> {
        Ok(body.replacen("schema_version: 1", "schema_version: 2", 1))
    }

    fn bump_two_to_three(body: &str) -> Result<String, String> {
        Ok(body.replacen("schema_version: 2", "schema_version: 3", 1))
    }

    fn broken_step(body: &str) -> Result<String, String> {
        Ok(body.to_owned())
    }

    fn failing_step(_body: &str) -> Result<String, String> {
        Err("cannot upgrade".to_owned())
    }

    const BODY_V1: &str = "schema_version: 1\nstate: |-\n  text\n";

    #[test]
    fn the_registry_is_empty_while_the_current_version_is_one() {
        assert_eq!(current_record_version(), 1);
        assert!(UPGRADE_STEPS.is_empty());
    }

    #[test]
    fn a_current_record_is_left_alone() {
        let result = upgrade_record_body(BODY_V1, &UPGRADE_STEPS, 1);

        assert_eq!(result, Ok((BODY_V1.to_owned(), 0)));
    }

    #[test]
    fn steps_run_in_order_from_the_recorded_version() {
        let steps = [
            VersionStep {
                from: 2,
                apply: bump_two_to_three,
            },
            VersionStep {
                from: 1,
                apply: bump_one_to_two,
            },
        ];

        let (body, applied) = upgrade_record_body(BODY_V1, &steps, 3).expect("upgrade");

        assert_eq!(page_version(&body), 3);
        assert_eq!(applied, 2);
    }

    #[test]
    fn a_record_newer_than_the_target_is_unsupported() {
        let result = upgrade_record_body("schema_version: 9\nstate: |-\n  text\n", &[], 1);

        assert_eq!(result, Err(SkipReason::UnsupportedVersion { found: 9 }));
    }

    #[test]
    fn a_missing_step_is_reported() {
        let result = upgrade_record_body(BODY_V1, &[], 2);

        assert_eq!(result, Err(SkipReason::MissingUpgradeStep { from: 1 }));
    }

    #[test]
    fn a_step_that_does_not_advance_the_version_is_rejected() {
        let steps = [VersionStep {
            from: 1,
            apply: broken_step,
        }];

        let result = upgrade_record_body(BODY_V1, &steps, 2);

        assert!(matches!(result, Err(SkipReason::InvalidRecordBody { .. })));
    }

    #[test]
    fn a_step_error_is_carried_into_the_skip_reason() {
        let steps = [VersionStep {
            from: 1,
            apply: failing_step,
        }];

        let result = upgrade_record_body(BODY_V1, &steps, 2);

        assert_eq!(
            result,
            Err(SkipReason::InvalidRecordBody {
                error: "cannot upgrade".to_owned()
            })
        );
    }
}
