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
diesel::joinable!(data_analysis -> datasets (dataset_id));
diesel::joinable!(datasets -> beamlines (beamline_id));
diesel::joinable!(datasets -> scan_types (scan_type_id));
diesel::joinable!(datasets -> syncotron_runs (syncotron_run_id));
diesel::joinable!(proposal_dataset_links -> datasets (dataset_id));
diesel::joinable!(proposal_dataset_links -> proposals (proposal_id));
diesel::joinable!(experimenter_proposal_links -> experiment_roles (experiment_role_id));
diesel::joinable!(experimenter_proposal_links -> proposals (proposal_id));
diesel::joinable!(experimenter_proposal_links -> users (user_badge));
diesel::joinable!(users -> user_access_controls (user_access_control_id));

diesel::allow_tables_to_appear_in_same_query!(
    beamline_contacts,
    beamlines,
    data_analysis,
    datasets,
    experiment_roles,
    experimenter_proposal_links,
    proposal_dataset_links,
    proposals,
    scan_types,
    syncotron_runs,
    user_access_controls,
    users,
);
