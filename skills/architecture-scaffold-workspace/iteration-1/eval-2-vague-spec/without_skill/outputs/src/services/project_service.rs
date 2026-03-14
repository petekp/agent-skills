use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::analytics::AnalyticsCollector;
use crate::models::*;
use crate::notifications::NotificationSink;

/// Manages projects: creation, membership, and statistics.
pub struct ProjectService {
    projects: HashMap<ProjectId, Project>,
    notifications: Arc<dyn NotificationSink>,
    analytics: Arc<dyn AnalyticsCollector>,
}

impl ProjectService {
    pub fn new(
        projects: HashMap<ProjectId, Project>,
        notifications: Arc<dyn NotificationSink>,
        analytics: Arc<dyn AnalyticsCollector>,
    ) -> Self {
        Self {
            projects,
            notifications,
            analytics,
        }
    }

    pub fn projects(&self) -> &HashMap<ProjectId, Project> {
        &self.projects
    }

    pub fn create(
        &mut self,
        name: String,
        owner: UserId,
    ) -> Result<ProjectId, TaskFlowError> {
        let id = Uuid::new_v4().to_string();
        let project = Project {
            id: id.clone(),
            name,
            description: None,
            owner: owner.clone(),
            members: vec![owner.clone()],
            created_at: Utc::now(),
        };
        self.projects.insert(id.clone(), project);
        self.analytics.track("project_created", Some(&owner), Some(&id));
        Ok(id)
    }

    pub fn add_member(
        &mut self,
        project_id: &ProjectId,
        user_id: UserId,
        actor: &UserId,
    ) -> Result<(), TaskFlowError> {
        let project = self.projects.get_mut(project_id)
            .ok_or_else(|| TaskFlowError::NotFound(format!("project: {}", project_id)))?;

        if &project.owner != actor {
            return Err(TaskFlowError::PermissionDenied {
                user: actor.clone(),
                action: "add member".to_string(),
            });
        }

        if !project.members.contains(&user_id) {
            project.members.push(user_id.clone());
            let name = project.name.clone();
            self.notifications.send(&Notification {
                kind: NotificationKind::ProjectInvite,
                user_id: user_id.clone(),
                message: format!("You were added to project: {}", name),
                timestamp: Utc::now(),
            });
        }

        self.analytics.track("member_added", Some(actor), Some(project_id));
        Ok(())
    }

    /// Check if a user is a member of a project.
    pub fn is_member(&self, user_id: &UserId, project_id: &ProjectId) -> bool {
        self.projects.get(project_id)
            .map_or(false, |p| p.members.contains(user_id))
    }

    /// Check if a user is the owner of a project.
    pub fn is_owner(&self, user_id: &UserId, project_id: &ProjectId) -> bool {
        self.projects.get(project_id)
            .map_or(false, |p| &p.owner == user_id)
    }

    /// Compute stats for a project given a slice of all tasks.
    pub fn stats(
        &self,
        project_id: &ProjectId,
        all_tasks: &HashMap<Uuid, Task>,
    ) -> Result<ProjectStats, TaskFlowError> {
        if !self.projects.contains_key(project_id) {
            return Err(TaskFlowError::NotFound(format!("project: {}", project_id)));
        }

        let project_tasks: Vec<&Task> = all_tasks.values()
            .filter(|t| &t.project_id == project_id)
            .collect();

        let total = project_tasks.len();
        let done = project_tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        let overdue = project_tasks.iter().filter(|t| {
            t.due_date.map_or(false, |d| d < Utc::now() && t.status != TaskStatus::Done)
        }).count();

        Ok(ProjectStats {
            total_tasks: total,
            completed_tasks: done,
            overdue_tasks: overdue,
            completion_rate: if total > 0 { done as f64 / total as f64 } else { 0.0 },
        })
    }
}
