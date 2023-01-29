use std::fmt;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MigrationId {
    pub app: String,
    pub name: String,
}

impl MigrationId {
    pub fn new(app: String, name: String) -> Self {
        Self { app, name }
    }

    pub fn try_from<S>(s: S) -> Option<Self>
    where
        S: AsRef<str>,
    {
        let mut iter = s.as_ref().split("::");
        let (app, name) = (iter.next()?.to_owned(), iter.next()?.to_owned());
        Some(Self { app, name })
    }
}

impl fmt::Display for MigrationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.app)?;
        f.write_str("::")?;
        f.write_str(&self.name)
    }
}
