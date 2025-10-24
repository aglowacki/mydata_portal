// @generated automatically by Diesel CLI.

diesel::table! {
    beamline_contacts (id) {
        user_badge -> Int4,
        beamline_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    beamlines (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 8]
        acronym -> Varchar,
        #[max_length = 8]
        old_acronym -> Nullable<Varchar>,
        #[max_length = 50]
        division -> Varchar,
        #[max_length = 400]
        link -> Varchar,
    }
}

diesel::table! {
    bio_sample_conditions (id) {
        id -> Int4,
        #[max_length = 512]
        name -> Varchar,
    }
}

diesel::table! {
    bio_sample_fixations (id) {
        id -> Int4,
        #[max_length = 512]
        name -> Varchar,
        fixative_id -> Int4,
    }
}

diesel::table! {
    bio_sample_fixatives (id) {
        id -> Int4,
        #[max_length = 512]
        name -> Varchar,
    }
}

diesel::table! {
    bio_sample_type_origin_sub_origin_links (id) {
        id -> Int4,
        bio_sample_type_id -> Int4,
        origin_id -> Int4,
        sub_origin_id -> Int4,
    }
}

diesel::table! {
    bio_sample_types (id) {
        id -> Int4,
        #[max_length = 512]
        type_name -> Varchar,
    }
}

diesel::table! {
    bio_samples (id) {
        id -> Int4,
        proposal_id -> Int4,
        #[max_length = 1000]
        name -> Varchar,
        type_id -> Int4,
        origin_id -> Int4,
        sub_origin_id -> Int4,
        source_id -> Int4,
        thickness -> Int4,
        #[max_length = 256]
        cell_line -> Nullable<Varchar>,
        is_cancer -> Bool,
        condition_id -> Int4,
        #[max_length = 2000]
        treatment_details -> Nullable<Varchar>,
        fixation_id -> Int4,
        #[max_length = 2000]
        expected_elemental_content_change -> Nullable<Varchar>,
        #[max_length = 3000]
        notes -> Nullable<Varchar>,
    }
}

diesel::table! {
    data_analysis (id) {
        id -> Int4,
        #[max_length = 2000]
        path -> Varchar,
        dataset_id -> Int4,
        analysis_submit_time -> Timestamp,
        processing_start_time -> Nullable<Timestamp>,
        processing_end_time -> Nullable<Timestamp>,
    }
}

diesel::table! {
    datasets (id) {
        id -> Int4,
        #[max_length = 2000]
        path -> Varchar,
        acquisition_timestamp -> Timestamp,
        beamline_id -> Int4,
        syncotron_run_id -> Int4,
        scan_type_id -> Int4,
    }
}

diesel::table! {
    experiment_roles (id) {
        id -> Int4,
        #[max_length = 255]
        role -> Varchar,
    }
}

diesel::table! {
    experimenter_proposal_links (id) {
        id -> Int4,
        user_badge -> Int4,
        proposal_id -> Int4,
        experiment_role_id -> Int4,
    }
}

diesel::table! {
    experimenters (id) {
        dataset_id -> Int4,
        user_badge -> Int4,
        proposal_id -> Int4,
        experiment_role_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    proposal_dataset_links (id) {
        id -> Int4,
        dataset_id -> Int4,
        proposal_id -> Int4,
    }
}

diesel::table! {
    proposals (id) {
        id -> Int4,
        #[max_length = 10000]
        title -> Varchar,
        #[max_length = 1]
        proprietaryflag -> Varchar,
        #[max_length = 1]
        mailinflag -> Varchar,
        #[max_length = 400]
        status -> Nullable<Varchar>,
    }
}

diesel::table! {
    sample_origins (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    sample_sources (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    sample_sub_origins (id) {
        id -> Int4,
        #[max_length = 2000]
        name -> Varchar,
    }
}

diesel::table! {
    scan_types (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 1000]
        description -> Nullable<Varchar>,
    }
}

diesel::table! {
    syncotron_runs (id) {
        id -> Int4,
        #[max_length = 6]
        name -> Bpchar,
        start_timestamp -> Timestamptz,
        end_timestamp -> Timestamptz,
    }
}

diesel::table! {
    user_access_controls (id) {
        id -> Int4,
        #[max_length = 10]
        level -> Varchar,
        #[max_length = 25]
        description -> Varchar,
    }
}

diesel::table! {
    users (badge) {
        badge -> Int4,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 50]
        first_name -> Varchar,
        #[max_length = 50]
        last_name -> Varchar,
        #[max_length = 400]
        institution -> Varchar,
        #[max_length = 50]
        email -> Varchar,
        user_access_control_id -> Int4,
    }
}

diesel::joinable!(beamline_contacts -> beamlines (beamline_id));
diesel::joinable!(beamline_contacts -> users (user_badge));
diesel::joinable!(bio_sample_fixations -> bio_sample_fixatives (fixative_id));
diesel::joinable!(bio_sample_type_origin_sub_origin_links -> bio_sample_types (bio_sample_type_id));
diesel::joinable!(bio_sample_type_origin_sub_origin_links -> sample_origins (origin_id));
diesel::joinable!(bio_sample_type_origin_sub_origin_links -> sample_sub_origins (sub_origin_id));
diesel::joinable!(bio_samples -> bio_sample_conditions (condition_id));
diesel::joinable!(bio_samples -> bio_sample_fixations (fixation_id));
diesel::joinable!(bio_samples -> bio_sample_types (type_id));
diesel::joinable!(bio_samples -> proposals (proposal_id));
diesel::joinable!(bio_samples -> sample_origins (origin_id));
diesel::joinable!(bio_samples -> sample_sources (source_id));
diesel::joinable!(bio_samples -> sample_sub_origins (sub_origin_id));
diesel::joinable!(data_analysis -> datasets (dataset_id));
diesel::joinable!(datasets -> beamlines (beamline_id));
diesel::joinable!(datasets -> scan_types (scan_type_id));
diesel::joinable!(datasets -> syncotron_runs (syncotron_run_id));
diesel::joinable!(experimenter_proposal_links -> experiment_roles (experiment_role_id));
diesel::joinable!(experimenter_proposal_links -> proposals (proposal_id));
diesel::joinable!(experimenter_proposal_links -> users (user_badge));
diesel::joinable!(experimenters -> datasets (dataset_id));
diesel::joinable!(experimenters -> experiment_roles (experiment_role_id));
diesel::joinable!(experimenters -> proposals (proposal_id));
diesel::joinable!(experimenters -> users (user_badge));
diesel::joinable!(proposal_dataset_links -> datasets (dataset_id));
diesel::joinable!(proposal_dataset_links -> proposals (proposal_id));
diesel::joinable!(users -> user_access_controls (user_access_control_id));

diesel::allow_tables_to_appear_in_same_query!(
    beamline_contacts,
    beamlines,
    bio_sample_conditions,
    bio_sample_fixations,
    bio_sample_fixatives,
    bio_sample_type_origin_sub_origin_links,
    bio_sample_types,
    bio_samples,
    data_analysis,
    datasets,
    experiment_roles,
    experimenter_proposal_links,
    experimenters,
    proposal_dataset_links,
    proposals,
    sample_origins,
    sample_sources,
    sample_sub_origins,
    scan_types,
    syncotron_runs,
    user_access_controls,
    users,
);
