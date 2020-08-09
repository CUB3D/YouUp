table! {
    projects (id) {
        id -> Integer,
        url -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        created -> Datetime,
    }
}
table! {
    status (id) {
        id -> Integer,
        project -> Integer,
        time -> Integer,
        status_code -> Integer,
        created -> Datetime,
    }
}
table! {
    incidents (id) {
        id -> Integer,
        created -> Datetime,
        project -> Integer,
    }
}

table! {
    incident_status_type (id) {
        id -> Integer,
        created -> Datetime,
        colour -> Varchar,
        title -> Varchar,
    }
}

table! {
    incident_status_update (id) {
        id -> Integer,
        created -> Datetime,
        status_type -> Integer,
        message -> Varchar,
        incident -> Integer,
    }
}

joinable!(incident_status_update -> incident_status_type(status_type));

allow_tables_to_appear_in_same_query!(incident_status_update, incident_status_type,);
