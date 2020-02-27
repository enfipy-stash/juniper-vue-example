use crate::{context::JuniperContext, models::User};
use futures::Stream;
use juniper::{DefaultScalarValue, EmptyMutation, FieldError, RootNode};
use std::{pin::Pin, time::Duration};

#[derive(Debug)]
pub struct Query;

#[juniper::graphql_object(Context = JuniperContext)]
impl Query {
    fn users(ctx: &JuniperContext) -> Vec<User> {
        ctx.database().get_all_users()
    }

    fn friends(ctx: &JuniperContext, id: i32) -> Vec<User> {
        ctx.database().get_friends(id)
    }
}

type TypeAlias = Pin<Box<dyn Stream<Item = Result<User, FieldError>> + Send>>;

#[derive(Debug)]
pub struct Subscription;

#[juniper::graphql_subscription(Context = JuniperContext)]
impl Subscription {
    async fn users(ctx: &JuniperContext) -> TypeAlias {
        let mut counter = 0;
        let database = ctx.database();
        let stream = tokio::time::interval(Duration::from_secs(5)).map(move |_| {
            database
                .get_user(counter)
                .ok_or(FieldError::new(
                    "some field error from handler",
                    Value::Scalar(DefaultScalarValue::String("some additional string".to_string())),
                ))
                .and_then(|res| {
                    counter += 1;
                    Ok(res)
                })
        });
        Box::pin(stream)
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<JuniperContext>, Subscription>;

pub fn init() -> Schema {
    Schema::new(Query, EmptyMutation::new(), Subscription)
}
