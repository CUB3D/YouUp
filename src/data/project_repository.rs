use crate::db::Database;
use crate::models::Project;
use crate::schema::projects;
use actix_web::web::Data;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub type ProjectRepositoryData = Data<Box<dyn ProjectRepository>>;

pub trait ProjectRepository {
    fn get_project_by_name(&self, name: &str) -> Vec<Project>;
    fn get_all_projects(&self) -> Vec<Project>;
    fn get_project_by_id(&self, id: i32) -> Option<Project>;
}

impl ProjectRepository for Database {
    fn get_project_by_name(&self, name: &str) -> Vec<Project> {
        projects::table
            .filter(projects::name.eq(name))
            .load::<Project>(&mut self.get().unwrap())
            .expect("Unable to load projects")
    }

    fn get_all_projects(&self) -> Vec<Project> {
        projects::table
            .load::<Project>(&mut self.get().unwrap())
            .expect("Unable to load projects")
    }

    fn get_project_by_id(&self, id: i32) -> Option<Project> {
        projects::table
            .filter(projects::id.eq(id))
            .load::<Project>(&mut self.get().unwrap())
            .expect("Unable to load project by id")
            .first()
            .cloned()
    }
}
