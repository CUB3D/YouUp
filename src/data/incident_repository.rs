use crate::db::Database;
use crate::diesel::Insertable;
use crate::models::{IncidentStatusType, IncidentStatusUpdate, Incidents, NewIncident};
use crate::schema::{incident_status_type, incident_status_update, incidents};
use actix_web::web::Data;
use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl};

pub type IncidentRepositoryData = Data<Box<dyn IncidentRepository>>;

pub trait IncidentRepository {
    fn get_incident_by_name(&self, name: &str) -> Vec<Incidents>;
    fn get_all_incidents(&self) -> Vec<Incidents>;

    fn add_incident(&self, incident: NewIncident);
    fn get_incident_by_id(&self, id: i32) -> Incidents;

    fn get_status_updates_by_incident(
        &self,
        incident: &Incidents,
    ) -> Vec<(IncidentStatusUpdate, IncidentStatusType)>;
}

impl IncidentRepository for Database {
    fn get_incident_by_name(&self, _name: &str) -> Vec<Incidents> {
        unimplemented!("get_incident_by_name not impl")
    }

    fn get_all_incidents(&self) -> Vec<Incidents> {
        incidents::table
            .load(&self.get().unwrap())
            .expect("Unable to get all incidents")
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

    fn get_status_updates_by_incident(
        &self,
        incident: &Incidents,
    ) -> Vec<(IncidentStatusUpdate, IncidentStatusType)> {
        IncidentStatusUpdate::belonging_to(incident)
            .order(incident_status_update::dsl::created.desc())
            .inner_join(incident_status_type::table)
            .filter(incident_status_type::dsl::id.eq(incident_status_update::dsl::id))
            .load(&self.get().unwrap())
            .expect("Unable to load status updates by incident")
    }
}
