use crate::db::Database;
use crate::diesel::Insertable;
use crate::models::{
    IncidentStatusType, IncidentStatusUpdate, Incidents, NewIncident, NewIncidentStatusUpdate,
};
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
    fn add_status_update(&self, status_update: NewIncidentStatusUpdate);

    fn get_incident_status_type_by_title(&self, name: &str) -> Option<IncidentStatusType>;
}

impl IncidentRepository for Database {
    fn get_incident_by_name(&self, _name: &str) -> Vec<Incidents> {
        unimplemented!("get_incident_by_name not impl")
    }

    fn get_all_incidents(&self) -> Vec<Incidents> {
        incidents::table
            .load(&mut self.get().unwrap())
            .expect("Unable to get all incidents")
    }

    fn add_incident(&self, incident: NewIncident) {
        incident
            .insert_into(incidents::table)
            .execute(&mut self.get().unwrap())
            .expect("Unable to insert incident");
    }

    //TODO: this should forward the option
    fn get_incident_by_id(&self, id: i32) -> Incidents {
        incidents::table
            .filter(incidents::id.eq(id))
            .load::<Incidents>(&mut self.get().unwrap())
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
            .filter(incident_status_type::dsl::id.eq(incident_status_update::dsl::status_type))
            .load(&mut self.get().unwrap())
            .expect("Unable to load status updates by incident")
    }

    fn add_status_update(&self, status_update: NewIncidentStatusUpdate) {
        status_update
            .insert_into(incident_status_update::table)
            .execute(&mut self.get().unwrap())
            .expect("Unable to insert incident status update");
    }

    fn get_incident_status_type_by_title(&self, name: &str) -> Option<IncidentStatusType> {
        incident_status_type::table
            .filter(incident_status_type::dsl::title.eq(name))
            .load::<IncidentStatusType>(&mut self.get().unwrap())
            .ok()
            .and_then(|u: Vec<IncidentStatusType>| u.first().cloned())
    }
}
