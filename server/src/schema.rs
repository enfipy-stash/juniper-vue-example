use futures::Stream;
use juniper::{DefaultScalarValue, EmptyMutation, FieldError, RootNode};
use std::{pin::Pin, time::Duration};

#[derive(Clone)]
pub struct Context {}

impl juniper::Context for Context {}

/// User representation
pub struct User {
    pub id: i32,
    pub name: String,
}

#[juniper::graphql_object(Context = Context)]
impl User {
    fn id(&self) -> i32 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn friends(&self) -> Vec<User> {
        if self.id == 1 {
            return vec![
                User {
                    id: 11,
                    name: "user11".into(),
                },
                User {
                    id: 12,
                    name: "user12".into(),
                },
                User {
                    id: 13,
                    name: "user13".into(),
                },
            ];
        } else if self.id == 2 {
            return vec![User {
                id: 21,
                name: "user21".into(),
            }];
        } else if self.id == 3 {
            return vec![
                User {
                    id: 31,
                    name: "user31".into(),
                },
                User {
                    id: 32,
                    name: "user32".into(),
                },
            ];
        } else {
            return vec![];
        }
    }
}

pub struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    async fn users(id: i32) -> Vec<User> {
        vec![User {
            id,
            name: "user1".into(),
        }]
    }
}

type TypeAlias = Pin<Box<dyn Stream<Item = Result<User, FieldError>> + Send>>;

pub struct Subscription;

#[juniper::graphql_subscription(Context = Context)]
impl Subscription {
    async fn users() -> TypeAlias {
        let mut counter = 0;
        let stream = tokio::time::interval(Duration::from_secs(5)).map(move |_| {
            counter += 1;
            if counter == 2 {
                Err(FieldError::new(
                    "some field error from handler",
                    Value::Scalar(DefaultScalarValue::String(
                        "some additional string".to_string(),
                    )),
                ))
            } else {
                Ok(User {
                    id: counter,
                    name: "stream user".to_string(),
                })
            }
        });
        Box::pin(stream)
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, Subscription>;

pub fn init() -> Schema {
    Schema::new(Query, EmptyMutation::new(), Subscription)
}
