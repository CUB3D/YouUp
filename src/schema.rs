table! {
    projects (id) {
        id -> Integer,
        url -> Varchar,
        name -> Varchar,
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
