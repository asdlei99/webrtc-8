use lazy_static::lazy_static;

use crate::{
    algorithms::{select_settings_candidates, SelectSettingsError},
    errors::OverconstrainedError,
    property::all::{name::*, names as all_properties},
    BareOrAdvancedMediaTrackConstraints, BareOrMandatoryMediaTrackConstraints,
    BareOrMediaTrackConstraints, FacingMode, MediaTrackSettings, MediaTrackSupportedConstraints,
    ResizeMode, ResolvedAdvancedMediaTrackConstraints, ResolvedMandatoryMediaTrackConstraints,
    ResolvedMediaTrackConstraint, ResolvedMediaTrackConstraintSet, ResolvedMediaTrackConstraints,
    ResolvedValueConstraint, ResolvedValueRangeConstraint, ResolvedValueSequenceConstraint,
    SanitizedMediaTrackConstraints,
};

use super::DeviceInformationExposureMode;

lazy_static! {
    static ref VIDEO_IDEAL: MediaTrackSettings = MediaTrackSettings::from_iter([
        (ASPECT_RATIO, 0.5625.into()),
        (FACING_MODE, FacingMode::user().into()),
        (FRAME_RATE, 60.0.into()),
        (WIDTH, 1920.into()),
        (HEIGHT, 1080.into()),
        (RESIZE_MODE, ResizeMode::none().into()),
    ]);
    static ref VIDEO_480P: MediaTrackSettings = MediaTrackSettings::from_iter([
        (DEVICE_ID, "480p".into()),
        (GROUP_ID, "builtin".into()),
        (ASPECT_RATIO, 0.5625.into()),
        (FACING_MODE, FacingMode::user().into()),
        (FRAME_RATE, 240.into()),
        (WIDTH, 720.into()),
        (HEIGHT, 480.into()),
        (RESIZE_MODE, ResizeMode::crop_and_scale().into()),
    ]);
    static ref VIDEO_720P: MediaTrackSettings = MediaTrackSettings::from_iter([
        (DEVICE_ID, "720p".into()),
        (GROUP_ID, "builtin".into()),
        (ASPECT_RATIO, 0.5625.into()),
        (FACING_MODE, FacingMode::user().into()),
        (FRAME_RATE, 120.into()),
        (WIDTH, 1280.into()),
        (HEIGHT, 720.into()),
        (RESIZE_MODE, ResizeMode::crop_and_scale().into()),
    ]);
    static ref VIDEO_1080P: MediaTrackSettings = MediaTrackSettings::from_iter([
        (DEVICE_ID, "1080p".into()),
        (GROUP_ID, "builtin".into()),
        (ASPECT_RATIO, 0.5625.into()),
        (FACING_MODE, FacingMode::user().into()),
        (FRAME_RATE, 60.into()),
        (WIDTH, 1920.into()),
        (HEIGHT, 1080.into()),
        (RESIZE_MODE, ResizeMode::none().into()),
    ]);
    static ref VIDEO_1440P: MediaTrackSettings = MediaTrackSettings::from_iter([
        (DEVICE_ID, "1440p".into()),
        (GROUP_ID, "builtin".into()),
        (ASPECT_RATIO, 0.5625.into()),
        (FACING_MODE, FacingMode::user().into()),
        (FRAME_RATE, 30.into()),
        (WIDTH, 2560.into()),
        (HEIGHT, 1440.into()),
        (RESIZE_MODE, ResizeMode::none().into()),
    ]);
    static ref VIDEO_2160P: MediaTrackSettings = MediaTrackSettings::from_iter([
        (DEVICE_ID, "2160p".into()),
        (GROUP_ID, "builtin".into()),
        (ASPECT_RATIO, 0.5625.into()),
        (FACING_MODE, FacingMode::user().into()),
        (FRAME_RATE, 15.into()),
        (WIDTH, 3840.into()),
        (HEIGHT, 2160.into()),
        (RESIZE_MODE, ResizeMode::none().into()),
    ]);
}

fn default_possible_settings() -> Vec<MediaTrackSettings> {
    vec![
        VIDEO_480P.clone(),
        VIDEO_720P.clone(),
        VIDEO_1080P.clone(),
        VIDEO_1440P.clone(),
        VIDEO_2160P.clone(),
    ]
}

fn default_supported_constraints() -> MediaTrackSupportedConstraints {
    MediaTrackSupportedConstraints::from_iter(all_properties())
}

fn test_overconstrained(
    possible_settings: &[MediaTrackSettings],
    mandatory_constraints: ResolvedMandatoryMediaTrackConstraints,
    exposure_mode: DeviceInformationExposureMode,
) -> OverconstrainedError {
    let constraints = ResolvedMediaTrackConstraints {
        mandatory: mandatory_constraints,
        advanced: ResolvedAdvancedMediaTrackConstraints::default(),
    }
    .to_sanitized(&default_supported_constraints());

    let result = select_settings_candidates(possible_settings.iter(), &constraints, exposure_mode);

    let actual = result.err().unwrap();

    let SelectSettingsError::Overconstrained(overconstrained_error) = actual;

    overconstrained_error
}

fn test_constrained(
    possible_settings: &[MediaTrackSettings],
    mandatory_constraints: ResolvedMandatoryMediaTrackConstraints,
    advanced_constraints: ResolvedAdvancedMediaTrackConstraints,
) -> Vec<&MediaTrackSettings> {
    let constraints = ResolvedMediaTrackConstraints {
        mandatory: mandatory_constraints,
        advanced: advanced_constraints,
    }
    .to_sanitized(&default_supported_constraints());

    let result = select_settings_candidates(
        possible_settings.iter(),
        &constraints,
        DeviceInformationExposureMode::Exposed,
    );

    result.unwrap()
}

mod unconstrained {
    use super::*;

    fn default_constraints() -> BareOrMediaTrackConstraints {
        BareOrMediaTrackConstraints {
            mandatory: BareOrMandatoryMediaTrackConstraints::default(),
            advanced: BareOrAdvancedMediaTrackConstraints::default(),
        }
    }

    fn default_resolved_constraints() -> ResolvedMediaTrackConstraints {
        default_constraints().into_resolved()
    }

    fn default_sanitized_constraints() -> SanitizedMediaTrackConstraints {
        default_resolved_constraints().into_sanitized(&default_supported_constraints())
    }

    #[test]
    fn pass_through() {
        let possible_settings = default_possible_settings();
        let sanitized_constraints = default_sanitized_constraints();

        let actual = select_settings_candidates(
            &possible_settings[..],
            &sanitized_constraints,
            DeviceInformationExposureMode::Exposed,
        )
        .unwrap();
        let expected: Vec<_> = possible_settings.iter().collect();

        assert_eq!(actual, expected);
    }
}

mod overconstrained {
    use super::*;

    #[test]
    fn protected() {
        let error = test_overconstrained(
            &default_possible_settings(),
            ResolvedMandatoryMediaTrackConstraints::from_iter([(
                GROUP_ID,
                ResolvedValueConstraint::default()
                    .exact("missing-group".to_owned())
                    .into(),
            )]),
            DeviceInformationExposureMode::Protected,
        );

        assert_eq!(error.constraint, "");
        assert_eq!(error.message, None);
    }

    mod exposed {
        use super::*;

        #[test]
        fn missing() {
            let error = test_overconstrained(
                &default_possible_settings(),
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    GROUP_ID,
                    ResolvedValueConstraint::default()
                        .exact("missing-group".to_owned())
                        .into(),
                )]),
                DeviceInformationExposureMode::Exposed,
            );

            let constraint = &error.constraint;
            let err_message = error.message.as_ref().expect("Error message.");

            assert_eq!(constraint, GROUP_ID);
            assert_eq!(
                err_message,
                "Setting was a mismatch ([\"builtin\"] do not satisfy (x == \"missing-group\"))."
            );
        }

        #[test]
        fn mismatch() {
            let error = test_overconstrained(
                &default_possible_settings(),
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    DEVICE_ID,
                    ResolvedValueConstraint::default()
                        .exact("mismatched-device".to_owned())
                        .into(),
                )]),
                DeviceInformationExposureMode::Exposed,
            );

            let constraint = &error.constraint;
            let err_message = error.message.as_ref().expect("Error message.");

            assert_eq!(constraint, DEVICE_ID);
            assert_eq!(
            err_message,
            "Setting was a mismatch ([\"1080p\", \"1440p\", \"2160p\", \"480p\", \"720p\"] do not satisfy (x == \"mismatched-device\"))."
        );
        }

        #[test]
        fn too_small() {
            let error = test_overconstrained(
                &default_possible_settings(),
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    FRAME_RATE,
                    ResolvedValueRangeConstraint::default().min(1000).into(),
                )]),
                DeviceInformationExposureMode::Exposed,
            );

            let constraint = &error.constraint;
            let err_message = error.message.as_ref().expect("Error message.");

            assert_eq!(constraint, FRAME_RATE);
            assert_eq!(
                err_message,
                "Setting was too small ([120, 15, 240, 30, 60] do not satisfy (1000 <= x))."
            );
        }

        #[test]
        fn too_large() {
            let error = test_overconstrained(
                &default_possible_settings(),
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    FRAME_RATE,
                    ResolvedValueRangeConstraint::default().max(10).into(),
                )]),
                DeviceInformationExposureMode::Exposed,
            );

            let constraint = &error.constraint;
            let err_message = error.message.as_ref().expect("Error message.");

            assert_eq!(constraint, FRAME_RATE);
            assert_eq!(
                err_message,
                "Setting was too large ([120, 15, 240, 30, 60] do not satisfy (x <= 10))."
            );
        }
    }
}

mod constrained {
    use super::*;

    #[test]
    fn specific_device_id() {
        let possible_settings = default_possible_settings();

        for target_settings in possible_settings.iter() {
            let setting = match target_settings.get(DEVICE_ID) {
                Some(setting) => setting,
                None => continue,
            };

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    DEVICE_ID,
                    ResolvedMediaTrackConstraint::exact_from(setting.clone()),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![target_settings];

            assert_eq!(actual, expected);
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn value() {
            let possible_settings = vec![
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "a".into()),
                    (GROUP_ID, "group-0".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "b".into()),
                    (GROUP_ID, "group-1".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "c".into()),
                    (GROUP_ID, "group-2".into()),
                ]),
            ];

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    GROUP_ID,
                    ResolvedValueConstraint::default()
                        .exact("group-1".to_owned())
                        .into(),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![&possible_settings[1]];

            assert_eq!(actual, expected);
        }

        #[test]
        fn value_range() {
            let possible_settings = vec![
                MediaTrackSettings::from_iter([(DEVICE_ID, "a".into()), (FRAME_RATE, 15.into())]),
                MediaTrackSettings::from_iter([(DEVICE_ID, "b".into()), (FRAME_RATE, 30.into())]),
                MediaTrackSettings::from_iter([(DEVICE_ID, "c".into()), (FRAME_RATE, 60.into())]),
            ];

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    FRAME_RATE,
                    ResolvedValueRangeConstraint::default().exact(30).into(),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![&possible_settings[1]];

            assert_eq!(actual, expected);
        }

        #[test]
        fn value_sequence() {
            let possible_settings = vec![
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "a".into()),
                    (GROUP_ID, "group-0".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "b".into()),
                    (GROUP_ID, "group-1".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "c".into()),
                    (GROUP_ID, "group-2".into()),
                ]),
            ];

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    GROUP_ID,
                    ResolvedValueSequenceConstraint::default()
                        .exact(vec!["group-1".to_owned(), "group-3".to_owned()])
                        .into(),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![&possible_settings[1]];

            assert_eq!(actual, expected);
        }
    }

    mod ideal {
        use super::*;

        #[test]
        fn value() {
            let possible_settings = vec![
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "a".into()),
                    (GROUP_ID, "group-0".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "b".into()),
                    (GROUP_ID, "group-1".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "c".into()),
                    (GROUP_ID, "group-2".into()),
                ]),
            ];

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    GROUP_ID,
                    ResolvedValueConstraint::default()
                        .ideal("group-1".to_owned())
                        .into(),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![&possible_settings[1]];

            assert_eq!(actual, expected);
        }

        #[test]
        fn value_range() {
            let possible_settings = vec![
                MediaTrackSettings::from_iter([(DEVICE_ID, "a".into()), (FRAME_RATE, 15.into())]),
                MediaTrackSettings::from_iter([(DEVICE_ID, "b".into()), (FRAME_RATE, 30.into())]),
                MediaTrackSettings::from_iter([(DEVICE_ID, "c".into()), (FRAME_RATE, 60.into())]),
            ];

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    FRAME_RATE,
                    ResolvedValueRangeConstraint::default().ideal(32).into(),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![&possible_settings[1]];

            assert_eq!(actual, expected);
        }

        #[test]
        fn value_sequence() {
            let possible_settings = vec![
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "a".into()),
                    (GROUP_ID, "group-0".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "b".into()),
                    (GROUP_ID, "group-1".into()),
                ]),
                MediaTrackSettings::from_iter([
                    (DEVICE_ID, "c".into()),
                    (GROUP_ID, "group-2".into()),
                ]),
            ];

            let actual = test_constrained(
                &possible_settings,
                ResolvedMandatoryMediaTrackConstraints::from_iter([(
                    GROUP_ID,
                    ResolvedValueSequenceConstraint::default()
                        .ideal(vec!["group-1".to_owned(), "group-3".to_owned()])
                        .into(),
                )]),
                ResolvedAdvancedMediaTrackConstraints::default(),
            );

            let expected = vec![&possible_settings[1]];

            assert_eq!(actual, expected);
        }
    }
}

// ```
//                        ┌
// mandatory constraints: ┤   ┄───────────────────────────────────────────┤
//                        └
//                        ┌
//  advanced constraints: ┤                    ├─┤         ├────────────────────────────┄
//                        └
//                        ┌
//     possible settings: ┤   ●─────────────●──────────────●──────────────●─────────────●
//                        └  480p          720p          1080p          1440p         2160p
//                                                         └───────┬──────┘
//     selected settings: ─────────────────────────────────────────┘
// ```
mod smoke {
    use super::*;

    #[test]
    fn native() {
        let supported_constraints =
            MediaTrackSupportedConstraints::from_iter(vec![DEVICE_ID, HEIGHT, WIDTH, RESIZE_MODE]);

        let possible_settings = vec![
            MediaTrackSettings::from_iter([
                (DEVICE_ID, "480p".into()),
                (HEIGHT, 480.into()),
                (WIDTH, 720.into()),
                (RESIZE_MODE, ResizeMode::crop_and_scale().into()),
            ]),
            MediaTrackSettings::from_iter([
                (DEVICE_ID, "720p".into()),
                (HEIGHT, 720.into()),
                (WIDTH, 1280.into()),
                (RESIZE_MODE, ResizeMode::crop_and_scale().into()),
            ]),
            MediaTrackSettings::from_iter([
                (DEVICE_ID, "1080p".into()),
                (HEIGHT, 1080.into()),
                (WIDTH, 1920.into()),
                (RESIZE_MODE, ResizeMode::none().into()),
            ]),
            MediaTrackSettings::from_iter([
                (DEVICE_ID, "1440p".into()),
                (HEIGHT, 1440.into()),
                (WIDTH, 2560.into()),
                (RESIZE_MODE, ResizeMode::none().into()),
            ]),
            MediaTrackSettings::from_iter([
                (DEVICE_ID, "2160p".into()),
                (HEIGHT, 2160.into()),
                (WIDTH, 3840.into()),
                (RESIZE_MODE, ResizeMode::none().into()),
            ]),
        ];

        let constraints: ResolvedMediaTrackConstraints = ResolvedMediaTrackConstraints {
            mandatory: ResolvedMandatoryMediaTrackConstraints::from_iter([
                (
                    WIDTH,
                    ResolvedValueRangeConstraint::default().max(2560).into(),
                ),
                (
                    HEIGHT,
                    ResolvedValueRangeConstraint::default().max(1440).into(),
                ),
                // Unsupported constraint, which should thus get ignored:
                (
                    FRAME_RATE,
                    ResolvedValueRangeConstraint::default().exact(30.0).into(),
                ),
            ]),
            advanced: ResolvedAdvancedMediaTrackConstraints::from_iter([
                // The first advanced constraint set of "exact 800p" does not match
                // any candidate and should thus get ignored by the algorithm:
                ResolvedMediaTrackConstraintSet::from_iter([(
                    HEIGHT,
                    ResolvedValueRangeConstraint::default().exact(800).into(),
                )]),
                // The second advanced constraint set of "no resizing" does match
                // candidates and should thus be applied by the algorithm:
                ResolvedMediaTrackConstraintSet::from_iter([(
                    RESIZE_MODE,
                    ResolvedValueConstraint::default()
                        .exact(ResizeMode::none())
                        .into(),
                )]),
            ]),
        };

        let sanitized_constraints = constraints.to_sanitized(&supported_constraints);

        let actual = select_settings_candidates(
            &possible_settings,
            &sanitized_constraints,
            DeviceInformationExposureMode::Exposed,
        )
        .unwrap();

        let expected = vec![&possible_settings[2], &possible_settings[3]];

        assert_eq!(actual, expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json() {
        let supported_constraints =
            MediaTrackSupportedConstraints::from_iter(vec![DEVICE_ID, HEIGHT, WIDTH, RESIZE_MODE]);

        // Deserialize possible settings from JSON:
        let possible_settings: Vec<MediaTrackSettings> = {
            let json = serde_json::json!([
                { "deviceId": "480p", "width": 720, "height": 480, "resizeMode": "crop-and-scale" },
                { "deviceId": "720p", "width": 1280, "height": 720, "resizeMode": "crop-and-scale" },
                { "deviceId": "1080p", "width": 1920, "height": 1080, "resizeMode": "none" },
                { "deviceId": "1440p", "width": 2560, "height": 1440, "resizeMode": "none" },
                { "deviceId": "2160p", "width": 3840, "height": 2160, "resizeMode": "none" },
            ]);
            serde_json::from_value(json).unwrap()
        };

        // Deserialize constraints from JSON:
        let bare_or_constraints: BareOrMediaTrackConstraints = {
            let json = serde_json::json!({
                "width": {
                    "max": 2560,
                },
                "height": {
                    "max": 1440,
                },
                // Unsupported constraint, which should thus get ignored:
                "frameRate": {
                    "exact": 30.0
                },
                "advanced": [
                    // The first advanced constraint set of "exact 800p" does not match
                    // any candidate and should thus get ignored by the algorithm:
                    { "height": 800 },
                    // The second advanced constraint set of "no resizing" does match
                    // candidates and should thus be applied by the algorithm:
                    { "resizeMode": "none" },
                ]
            });
            serde_json::from_value(json).unwrap()
        };

        // Resolve bare values to proper constraints:
        let constraints = bare_or_constraints.into_resolved();

        // Sanitize constraints, removing empty and unsupported constraints:
        let sanitized_constraints = constraints.to_sanitized(&supported_constraints);

        let actual = select_settings_candidates(
            &possible_settings,
            &sanitized_constraints,
            DeviceInformationExposureMode::Exposed,
        )
        .unwrap();

        let expected = vec![&possible_settings[2], &possible_settings[3]];

        assert_eq!(actual, expected);
    }
}
