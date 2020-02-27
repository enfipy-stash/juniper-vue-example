use crate::models::User;
use rand::Rng;

#[derive(Debug)]
pub struct Database {
    users: Vec<User>,
}

impl Database {
    pub fn new() -> Self {
        let mut users = vec![];
        for i in 0..5 {
            users.push(User {
                id: i,
                name: format!("user{}", i),
                friends: vec![],
            });
        }
        let mut rng = rand::thread_rng();
        let users_temp = users.clone();
        for user in users.iter_mut() {
            user.friends = users_temp
                .clone()
                .into_iter()
                .take_while(|user| user.id < rng.gen_range(0, 5))
                .collect();
        }
        Self { users }
    }

    pub fn get_user(&self, id: i32) -> Option<User> {
        self.users.clone().into_iter().find(|user| user.id == id)
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.users.clone()
    }

    pub fn get_friends(&self, id: i32) -> Vec<User> {
        self.users
            .clone()
            .into_iter()
            .find(|user| user.id == id)
            .map_or(vec![], |user| {
                user.friends
                    .iter()
                    .map(|user| self.get_user(user.id).unwrap())
                    .collect()
            })
    }
}
