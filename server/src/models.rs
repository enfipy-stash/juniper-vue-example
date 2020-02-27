#[derive(Clone, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub friends: Vec<User>,
}

#[juniper::graphql_object]
impl User {
    fn id(&self) -> i32 {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn friends(&self) -> Vec<User> {
        self.friends.clone()
    }
}
