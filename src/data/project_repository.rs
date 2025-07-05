use crate::db::Database;
use crate::diesel::Insertable;
use crate::models::{NewProject, Project};
use crate::schema::projects;
use actix_web::web::Data;
use anyhow::Context;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub type ProjectRepositoryData = Data<Box<dyn ProjectRepository>>;

pub trait ProjectRepository {
    fn get_project_by_name(&self, name: &str) -> anyhow::Result<Vec<Project>>;
    fn get_all_enabled_projects(&self) -> anyhow::Result<Vec<Project>>;
    fn get_project_by_id(&self, id: i32) -> Option<Project>;
    fn create(&self, name: &str) -> anyhow::Result<()>;
    fn update_project(
        &self,
        id: i32,
        name: &str,
        enable: bool,
        url: &str,
        description: &str,
    ) -> anyhow::Result<()>;
}

impl ProjectRepository for Database {
    fn get_project_by_name(&self, name: &str) -> anyhow::Result<Vec<Project>> {
        let mut pool = self.get()?;

        Ok(projects::table
            .filter(projects::name.eq(name))
            .load::<Project>(&mut pool)
            .expect("Unable to load projects"))
    }

    fn get_all_enabled_projects(&self) -> anyhow::Result<Vec<Project>> {
        let mut pool = self.get()?;

        Ok(projects::table
            .filter(projects::enabled.eq(true))
            .load::<Project>(&mut pool)
            .expect("Unable to load projects"))
    }

    fn get_project_by_id(&self, id: i32) -> Option<Project> {
        projects::table
            .filter(projects::id.eq(id))
            .load::<Project>(&mut self.get().unwrap())
            .expect("Unable to load project by id")
            .first()
            .cloned()
    }

    fn create(&self, name: &str) -> anyhow::Result<()> {
        NewProject {
            url: "".to_string(),
            name: name.to_string(),
            description: None,
            enabled: false,
        }
        .insert_into(projects::table)
        .execute(&mut self.get()?)
        .context("Unable to insert project")?;
        Ok(())
    }

    fn update_project(
        &self,
        id: i32,
        name: &str,
        enable: bool,
        url: &str,
        description: &str,
    ) -> anyhow::Result<()> {
        diesel::update(projects::table)
            .filter(projects::id.eq(id))
            .set((
                projects::name.eq(name),
                projects::enabled.eq(enable),
                projects::description.eq(description),
                projects::url.eq(url),
            ))
            .execute(&mut self.get()?)
            .context("Failed to update project")?;
        Ok(())
    }
}
