use async_graphql::*;
use futures::{Stream, StreamExt};

#[derive(SimpleObject)]
struct Object1 {
    a: i32,
}

#[derive(SimpleObject)]
struct Object2 {
    b: i32,
}

#[derive(SimpleObject)]
struct Object3 {
    c: i32,
}

#[async_std::test]
pub async fn test_merged_object() {
    type MyObj =
        MergedObject<Object1, MergedObject<Object2, MergedObject<Object3, MergedObjectTail>>>;

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MergedObject(
                Object1 { a: 10 },
                MergedObject(
                    Object2 { b: 20 },
                    MergedObject(Object3 { c: 30 }, MergedObjectTail),
                ),
            )
        }
    }

    assert_eq!(
        MyObj::type_name(),
        "Object1_Object2_Object3_MergedObjectTail"
    );

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}

#[async_std::test]
pub async fn test_merged_object_macro() {
    #[derive(MergedObject)]
    struct MyObj(Object1, Object2, Object3);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj(Object1 { a: 10 }, Object2 { b: 20 }, Object3 { c: 30 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}

#[async_std::test]
pub async fn test_merged_object_derive() {
    #[derive(MergedObject)]
    struct MyObj(Object1, Object2, Object3);

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj(Object1 { a: 10 }, Object2 { b: 20 }, Object3 { c: 30 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}

#[async_std::test]
pub async fn test_merged_object_default() {
    mod a {
        use super::*;

        #[derive(SimpleObject)]
        pub struct QueryA {
            pub a: i32,
        }

        impl Default for QueryA {
            fn default() -> Self {
                Self { a: 10 }
            }
        }
    }

    mod b {
        use super::*;

        #[derive(SimpleObject)]
        pub struct QueryB {
            pub b: i32,
        }

        impl Default for QueryB {
            fn default() -> Self {
                Self { b: 20 }
            }
        }
    }

    #[derive(MergedObject, Default)]
    struct Query(a::QueryA, b::QueryB);

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    let query = "{ a b }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "a": 10,
            "b": 20,
        })
    );
}

#[async_std::test]
pub async fn test_merged_subscription() {
    #[derive(Default)]
    struct Subscription1;

    #[Subscription]
    impl Subscription1 {
        async fn events1(&self) -> impl Stream<Item = i32> {
            futures::stream::iter(0..10)
        }
    }

    #[derive(Default)]
    struct Subscription2;

    #[Subscription]
    impl Subscription2 {
        async fn events2(&self) -> impl Stream<Item = i32> {
            futures::stream::iter(10..20)
        }
    }

    #[derive(MergedSubscription, Default)]
    struct Subscription(Subscription1, Subscription2);

    struct Query;

    #[Object]
    impl Query {}

    let schema = Schema::new(Query, EmptyMutation, Subscription::default());

    {
        let mut stream = schema
            .execute_stream("subscription { events1 }")
            .map(|resp| resp.into_result().unwrap().data)
            .boxed();
        for i in 0i32..10 {
            assert_eq!(
                Some(serde_json::json!({
                    "events1": i,
                })),
                stream.next().await
            );
        }
        assert!(stream.next().await.is_none());
    }

    {
        let mut stream = schema
            .execute_stream("subscription { events2 }")
            .map(|resp| resp.into_result().unwrap().data)
            .boxed();
        for i in 10i32..20 {
            assert_eq!(
                Some(serde_json::json!({
                    "events2": i,
                })),
                stream.next().await
            );
        }
        assert!(stream.next().await.is_none());
    }
}
