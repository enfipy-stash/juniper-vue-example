use crate::{context::GraphqlContext, models::User};
use futures::Stream;
use juniper::{DefaultScalarValue, EmptyMutation, FieldError, RootNode};
use std::{pin::Pin, sync::atomic::Ordering, time::Duration};

#[derive(Debug)]
pub struct Query;

#[juniper::graphql_object(Context = GraphqlContext)]
impl Query {
    fn users(ctx: &GraphqlContext) -> Vec<User> {
        ctx.database().get_all_users()
    }

    fn friends(ctx: &GraphqlContext, id: i32) -> Vec<User> {
        ctx.database().get_friends(id)
    }
}

type TypeAlias = Pin<Box<dyn Stream<Item = Result<User, FieldError>> + Send>>;

#[derive(Debug)]
pub struct Subscription;

#[juniper::graphql_subscription(Context = GraphqlContext)]
impl Subscription {
    async fn users(ctx: &GraphqlContext) -> TypeAlias {
        let mut counter = 0;
        let database = ctx.database();
        let received_ws_close_signal = ctx.received_ws_close_signal();
        let stream = tokio::time::interval(Duration::from_secs(5))
            .map(move |_| {
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
            })
            .take_while(move |_| {
                let closed = received_ws_close_signal.load(Ordering::Relaxed);
                if closed {
                    log::info!("closed_event");
                }
                async move { !closed }
            });
        Box::pin(stream)
    }
}

pub type GraphqlRoot = RootNode<'static, Query, EmptyMutation<GraphqlContext>, Subscription>;

pub fn init() -> GraphqlRoot {
    GraphqlRoot::new(Query, EmptyMutation::new(), Subscription)
}
