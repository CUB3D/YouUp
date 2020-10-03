use crate::db::Database;
use crate::diesel::Insertable;
use crate::models::{Incidents, NewIncident};
use crate::schema::incidents;
use actix_web::web::Data;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub type IncidentRepositoryData = Data<Box<dyn IncidentRepository>>;

pub trait IncidentRepository {
    fn get_incident_by_name(&self, name: &str) -> Vec<Incidents>;
    fn get_all_incidents(&self) -> Vec<Incidents>;

    fn add_incident(&self, incident: NewIncident);
    fn get_incident_by_id(&self, id: i32) -> Incidents;
}

impl IncidentRepository for Database {
    fn get_incident_by_name(&self, name: &str) -> Vec<Incidents> {
        unimplemented!()
    }

    fn get_all_incidents(&self) -> Vec<Incidents> {
        unimplemented!()
    }

    fn add_incident(&self, incident: NewIncident) {
        incident
            .insert_into(incidents::table)
            .execute(&self.get().unwrap())
            .expect("Unable to insert incident");
    }

    //TODO: this should forward the option
    fn get_incident_by_id(&self, id: i32) -> Incidents {
        incidents::table
            .filter(incidents::id.eq(id))
            .load::<Incidents>(&self.get().unwrap())
            .expect("Unable to load incidents")
            .first()
            .cloned()
            .unwrap()
    }
}
