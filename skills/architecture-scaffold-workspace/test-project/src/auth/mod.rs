use crate::domain::{Project, UserId};
pub trait AuthPolicy {
    fn can_create_task(&self, user: &UserId, project: &Project) -> bool;
    fn can_update_task(&self, user: &UserId, project: &Project) -> bool;
    fn can_delete_task(&self, user: &UserId, project: &Project) -> bool;
    fn can_assign_task(&self, user: &UserId, project: &Project) -> bool;
    fn can_add_member(&self, user: &UserId, project: &Project) -> bool;
}
pub struct DefaultAuthPolicy;
impl AuthPolicy for DefaultAuthPolicy {
    fn can_create_task(&self, _user: &UserId, _project: &Project) -> bool { todo!() }
    fn can_update_task(&self, _user: &UserId, _project: &Project) -> bool { todo!() }
    fn can_delete_task(&self, _user: &UserId, _project: &Project) -> bool { todo!() }
    fn can_assign_task(&self, _user: &UserId, _project: &Project) -> bool { todo!() }
    fn can_add_member(&self, _user: &UserId, _project: &Project) -> bool { todo!() }
}
