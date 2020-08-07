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
        status -> Varchar,
        message -> Varchar,
        project -> Integer,
    }
}
