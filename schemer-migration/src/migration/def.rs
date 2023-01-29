use std::{borrow::Cow, collections::HashSet, sync::Arc};

use super::id::MigrationId;

pub trait Migration: Send + Sync {
    fn name(&self) -> Cow<'static, str>;
    fn description(&self) -> &'static str;

    fn dependencies(&self) -> HashSet<MigrationId> {
        HashSet::new()
    }

    fn up(&self) -> Cow<'static, str>;
    fn down(&self) -> Cow<'static, str>;
}

impl<T: Migration + ?Sized> Migration for Box<T> {
    fn name(&self) -> Cow<'static, str> {
        self.as_ref().name()
    }

    fn description(&self) -> &'static str {
        self.as_ref().description()
    }

    fn dependencies(&self) -> HashSet<MigrationId> {
        self.as_ref().dependencies()
    }

    fn up(&self) -> Cow<'static, str> {
        self.as_ref().up()
    }

    fn down(&self) -> Cow<'static, str> {
        self.as_ref().down()
    }
}

#[derive(Clone)]
pub struct AppMigration {
    pub app: Cow<'static, str>,
    pub migration: Arc<dyn Migration>,
}

impl AppMigration {
    pub fn new<M: Migration + 'static>(app: Cow<'static, str>, migration: M) -> Self {
        Self {
            app,
            migration: Arc::new(migration),
        }
    }

    pub fn id(&self) -> MigrationId {
        MigrationId::new(self.app.to_string(), self.migration.name().to_string())
    }

    pub fn dependencies(&self) -> HashSet<MigrationId> {
        self.migration.dependencies()
    }

    pub fn description(&self) -> &'static str {
        self.migration.description()
    }

    pub fn app(&self) -> Cow<'static, str> {
        self.app.clone()
    }

    pub fn name(&self) -> Cow<'static, str> {
        self.migration.name()
    }

    pub fn up(&self) -> Cow<'static, str> {
        self.migration.up()
    }

    pub fn down(&self) -> Cow<'static, str> {
        self.migration.down()
    }
}

impl schemer::Migration<MigrationId> for AppMigration {
    fn id(&self) -> MigrationId {
        self.id()
    }

    fn dependencies(&self) -> HashSet<MigrationId> {
        self.dependencies()
    }

    fn description(&self) -> &'static str {
        self.description()
    }
}
