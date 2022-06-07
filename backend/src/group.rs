use std::borrow::Cow;

use anyhow::{Error, Result};
use dashmap::DashMap;
use futures::Future;
use once_cell::sync::Lazy;
use task_group::{TaskGroup, TaskManager};

pub static GROUPS: Lazy<Groups> = Lazy::new(|| Groups {
    groups: Default::default(),
});

pub trait Group {
    fn name(&self) -> Cow<'static, str>;
}

pub struct Groups {
    groups: DashMap<Cow<'static, str>, (TaskGroup<Error>, TaskManager<Error>)>,
}

impl Groups {
    pub async fn spawn_into(
        &self,
        group: &impl Group,
        future: impl Future<Output = Result<()>> + Send + 'static,
    ) -> Result<()> {
        let key = group.name();
        let stuff = self.groups.entry(key).or_insert_with(|| TaskGroup::new()).downgrade();
        Ok(stuff.0.spawn("", future).await?)
    }

    pub fn kill(
        &self,
        group: &impl Group,
    ) {
        self.groups.remove(&group.name());
    }
}
